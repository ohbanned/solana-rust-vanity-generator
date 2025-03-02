use cursive::align::HAlign;
use cursive::traits::*;
use cursive::views::{Dialog, EditView, LinearLayout, ProgressBar, RadioGroup, TextView};
use cursive::{Cursive, CursiveExt};
use serde_json::Value;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    // Create a basic Cursive app
    let mut siv = Cursive::default();

    // Set a simple theme
    siv.set_theme(cursive::theme::Theme::default());

    // Add a title and the ASCII art
    siv.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Solana Vanity Address Generator").h_align(HAlign::Center))
                .child(TextView::new(""))
                .child(
                    TextView::new(
                        "Generate Solana wallet addresses with custom prefixes or suffixes",
                    )
                    .h_align(HAlign::Center),
                )
                .child(TextView::new(""))
                .child(TextView::new("Built by Ban Github: @ohbanned").h_align(HAlign::Center))
                .child(TextView::new("")),
        )
        .button("Start Generator", |s| {
            main_form(s);
        })
        .button("Quit", |s| s.quit())
        .title("Solana Vanity Address Generator"),
    );

    siv.run();
}

fn main_form(siv: &mut Cursive) {
    // Dialog for input parameters
    let mut position_group = RadioGroup::new();
    siv.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Enter the pattern to search for:"))
                .child(
                    EditView::new()
                        .content("abc")
                        .with_name("pattern")
                        .fixed_width(20),
                )
                .child(TextView::new("\nSelect position:"))
                .child(
                    LinearLayout::horizontal()
                        .child(position_group.button_str("Prefix"))
                        .child(TextView::new(" "))
                        .child(position_group.button_str("Suffix")),
                )
                .child(TextView::new("\nServer Address (default is fine):"))
                .child(
                    EditView::new()
                        .content("http://127.0.0.1:3001")
                        .with_name("server")
                        .fixed_width(30),
                ),
        )
        .title("Generation Parameters")
        .button("Generate", move |s| {
            let pattern = s
                .call_on_name("pattern", |view: &mut EditView| view.get_content())
                .unwrap();

            let position = if position_group.selected_id() == 0 {
                "prefix"
            } else {
                "suffix"
            };

            let server = s
                .call_on_name("server", |view: &mut EditView| view.get_content())
                .unwrap();

            // Check that pattern is valid
            if pattern.len() < 3 || pattern.len() > 8 {
                s.add_layer(Dialog::info(
                    "Pattern must be between 3 and 8 characters long.",
                ));
                return;
            }

            // Launch the generation process
            generate_address(
                s,
                server.to_string(),
                pattern.to_string(),
                position.to_string(),
            );
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

async fn start_job(
    server: String,
    pattern: String,
    position: String,
) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post(&format!("{}/generate", server))
        .json(&serde_json::json!({
            "pattern": pattern,
            "position": position,
        }))
        .send()
        .await?;

    let json: Value = res.json().await?;
    Ok(json["job_id"].as_str().unwrap_or("").to_string())
}

async fn check_job_status(
    server: String,
    job_id: String,
) -> Result<(String, Option<(String, String)>), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .get(&format!("{}/status/{}", server, job_id))
        .send()
        .await?;

    let json: Value = res.json().await?;

    if let Some(status) = json.get("status") {
        let status_str = status.as_str().unwrap_or("");

        if status_str == "complete" {
            if let Some(result) = json.get("result") {
                let pub_key = result["public_key"].as_str().unwrap_or("").to_string();
                let priv_key = result["private_key"].as_str().unwrap_or("").to_string();
                return Ok((status_str.to_string(), Some((pub_key, priv_key))));
            }
        }

        return Ok((status_str.to_string(), None));
    }

    Ok(("error".to_string(), None))
}

fn generate_address(siv: &mut Cursive, server: String, pattern: String, position: String) {
    // Create a progress dialog
    siv.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(format!(
                    "Generating address with {} '{}'...",
                    position, pattern
                )))
                .child(TextView::new(""))
                .child(ProgressBar::new().with_name("progress"))
                .child(TextView::new("").with_name("status"))
                .child(TextView::new("")),
        )
        .title("Address Generation in Progress")
        .button("Cancel Job", move |s| {
            s.pop_layer();
            s.add_layer(Dialog::info("Job canceled."));
        }),
    );

    // We'll use a channel to communicate between the thread and the UI
    let (sender, receiver) = mpsc::channel();

    // Clone values for the thread
    let server_clone = server.clone();
    let pattern_clone = pattern.clone();
    let position_clone = position.clone();

    // Create a thread to do the actual generation
    thread::spawn(move || {
        // We need to create a runtime since we're in a thread
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            // Start the job
            match start_job(server_clone.clone(), pattern_clone, position_clone).await {
                Ok(job_id) => {
                    if job_id.is_empty() {
                        sender
                            .send(("error".to_string(), "Failed to start job".to_string(), None))
                            .unwrap();
                        return;
                    }

                    let start_time = Instant::now();

                    // Poll for results
                    loop {
                        match check_job_status(server_clone.clone(), job_id.clone()).await {
                            Ok((status, result)) => {
                                if status == "complete" {
                                    if let Some((pub_key, priv_key)) = result {
                                        let elapsed = start_time.elapsed().as_secs_f32();
                                        sender
                                            .send((
                                                "complete".to_string(),
                                                format!("Found in {:.2}s", elapsed),
                                                Some((pub_key, priv_key)),
                                            ))
                                            .unwrap();
                                        break;
                                    }
                                } else if status == "error" {
                                    sender
                                        .send((
                                            "error".to_string(),
                                            "Error checking job status".to_string(),
                                            None,
                                        ))
                                        .unwrap();
                                    break;
                                } else {
                                    // Still running - update with time elapsed
                                    let elapsed = start_time.elapsed().as_secs_f32();
                                    sender
                                        .send((
                                            "running".to_string(),
                                            format!("Running... {:.2}s elapsed", elapsed),
                                            None,
                                        ))
                                        .unwrap();
                                }
                            }
                            Err(e) => {
                                sender
                                    .send(("error".to_string(), format!("Error: {}", e), None))
                                    .unwrap();
                                break;
                            }
                        }

                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
                Err(e) => {
                    sender
                        .send(("error".to_string(), format!("Error: {}", e), None))
                        .unwrap();
                }
            }
        });
    });

    // Create a callback that will update the interface
    siv.set_autorefresh(true);
    siv.add_global_callback('q', move |s| s.quit());

    // Set up a callback to check for messages from the thread
    siv.cb_sink()
        .send(Box::new(move |s| {
            if let Ok((status, message, keys)) = receiver.try_recv() {
                if status == "complete" {
                    if let Some((pub_key, priv_key)) = keys {
                        // Found a match! Show the results
                        s.pop_layer(); // Remove progress dialog
                        s.add_layer(
                            Dialog::around(
                                LinearLayout::vertical()
                                    .child(
                                        TextView::new("✅ Address found!")
                                            .style(cursive::theme::Effect::Bold),
                                    )
                                    .child(TextView::new(message))
                                    .child(TextView::new(""))
                                    .child(
                                        TextView::new("📝 PUBLIC KEY:")
                                            .style(cursive::theme::Effect::Bold),
                                    )
                                    .child(TextView::new(pub_key.clone()))
                                    .child(TextView::new(""))
                                    .child(
                                        TextView::new("🔑 PRIVATE KEY:")
                                            .style(cursive::theme::Effect::Bold),
                                    )
                                    .child(TextView::new(priv_key.clone()))
                                    .child(TextView::new(""))
                                    .child(
                                        TextView::new(
                                            "⚠️ IMPORTANT: Save your private key securely!",
                                        )
                                        .style(cursive::theme::Effect::Bold),
                                    ),
                            )
                            .title("Vanity Address Generated")
                            .button("Generate Another", |s| {
                                s.pop_layer();
                                main_form(s);
                            })
                            .button("Quit", |s| s.quit()),
                        );
                        s.set_autorefresh(false);
                    }
                } else if status == "error" {
                    // Show error
                    s.pop_layer(); // Remove progress dialog
                    s.add_layer(
                        Dialog::around(TextView::new(format!("❌ Error: {}", message)))
                            .title("Error")
                            .button("OK", |s| {
                                s.pop_layer();
                                main_form(s);
                            }),
                    );
                    s.set_autorefresh(false);
                } else {
                    // Update the status
                    s.call_on_name("status", |view: &mut TextView| {
                        view.set_content(message);
                    });
                }
            }
        }))
        .unwrap();
}
