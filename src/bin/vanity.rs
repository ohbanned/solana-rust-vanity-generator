use colored::*;
use reqwest;
use serde_json::Value;
use std::env;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Check for proper usage
    if args.len() < 3 {
        println!("{}", "Usage:".yellow().bold());
        println!("{} <pattern> <position>", args[0].green());
        println!();
        println!("Example:");
        println!("{} abc prefix", args[0].green());
        println!("{} xyz suffix", args[0].green());
        println!();
        println!("Parameters:");
        println!(
            "  {} - 3-8 character pattern to search for",
            "pattern".cyan()
        );
        println!("  {} - Either 'prefix' or 'suffix'", "position".cyan());
        return Ok(());
    }

    let pattern = &args[1];
    let position = &args[2].to_lowercase();

    // Validate inputs
    if pattern.len() < 3 || pattern.len() > 8 {
        println!(
            "{} Pattern must be between 3-8 characters long",
            "ERROR:".red().bold()
        );
        return Ok(());
    }

    if position != "prefix" && position != "suffix" {
        println!(
            "{} Position must be 'prefix' or 'suffix'",
            "ERROR:".red().bold()
        );
        return Ok(());
    }

    // Set the server URL
    let server = "http://127.0.0.1:3001";

    println!(
        "{} Generating Solana address with {} '{}'...",
        "⏳".yellow(),
        position.cyan(),
        pattern.cyan().bold()
    );

    // Start the job
    let job_id = match start_job(server, pattern, position).await {
        Ok(id) => {
            if id.is_empty() {
                println!("{} Failed to start generation job", "ERROR:".red().bold());
                return Ok(());
            }
            id
        }
        Err(e) => {
            println!("{} {}", "ERROR:".red().bold(), e);
            println!(
                "Is the server running? Start it with {}",
                "./run_server.sh".green()
            );
            return Ok(());
        }
    };

    // Track time
    let start_time = Instant::now();
    let mut dots = 0;

    // Poll for results
    loop {
        match check_job_status(server, &job_id).await {
            Ok((status, result)) => {
                if status == "complete" {
                    if let Some((pub_key, priv_key)) = result {
                        let elapsed = start_time.elapsed().as_secs_f32();
                        println!(
                            "\n{} Address found in {:.2} seconds!",
                            "✅".green(),
                            elapsed
                        );
                        println!("\n{}", "PUBLIC KEY:".green().bold());
                        println!("{}", pub_key);
                        println!("\n{}", "PRIVATE KEY:".yellow().bold());
                        println!("{}", priv_key);
                        println!(
                            "\n{}",
                            "⚠️  IMPORTANT: Save your private key securely! ⚠️"
                                .red()
                                .bold()
                        );
                        break;
                    }
                } else if status == "error" {
                    println!("\n{} Error checking job status", "ERROR:".red().bold());
                    break;
                } else {
                    // Show progress indicator
                    print!(
                        "\r{} Searching{} elapsed: {:.1}s",
                        "⏳".yellow(),
                        ".".repeat(dots % 4 + 1),
                        start_time.elapsed().as_secs_f32()
                    );
                    dots += 1;
                    // Flush stdout to make sure the progress shows immediately
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                }
            }
            Err(e) => {
                println!("\n{} {}", "ERROR:".red().bold(), e);
                break;
            }
        }

        sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

async fn start_job(server: &str, pattern: &str, position: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/generate", server))
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
    server: &str,
    job_id: &str,
) -> Result<(String, Option<(String, String)>), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/status/{}", server, job_id))
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
