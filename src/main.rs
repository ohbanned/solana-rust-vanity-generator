use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use dashmap::DashMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, Signer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

// Structures for request and response
#[derive(Debug, Deserialize)]
struct GenerateRequest {
    pattern: String,
    position: String, // "prefix" or "suffix"
}

#[derive(Debug, Serialize)]
struct GenerateResponse {
    job_id: String,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String, // "pending", "running", "complete", "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<AddressResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
struct AddressResult {
    public_key: String,
    private_key: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
}

// Internal job tracking
struct Job {
    status: String,
    #[allow(dead_code)]
    pattern: String,
    #[allow(dead_code)]
    position: String,
    start_time: Instant,
    cancel_flag: Arc<AtomicBool>,
    result: Option<AddressResult>,
    error: Option<String>,
}

// Global state
struct AppState {
    jobs: DashMap<String, Arc<Mutex<Job>>>,
}

// Generate a vanity address
async fn generate_address(
    req: web::Json<GenerateRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let pattern = req.pattern.clone();
    let position = req.position.clone();

    // Validate the pattern and position
    if pattern.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Pattern cannot be empty"
        }));
    }

    if position != "prefix" && position != "suffix" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Position must be 'prefix' or 'suffix'"
        }));
    }

    // Create a new job
    let job_id = Uuid::new_v4().to_string();
    let cancel_flag = Arc::new(AtomicBool::new(false));

    let job = Job {
        status: "pending".to_string(),
        pattern: pattern.clone(),
        position: position.clone(),
        start_time: Instant::now(),
        cancel_flag: cancel_flag.clone(),
        result: None,
        error: None,
    };

    // Store the job
    data.jobs.insert(job_id.clone(), Arc::new(Mutex::new(job)));

    // Launch background task to find the address
    let job_ref = data.jobs.get(&job_id).unwrap().clone();
    let pattern_clone = pattern.clone();
    let position_clone = position.clone();

    tokio::spawn(async move {
        // Update job status to running
        {
            let mut job = job_ref.lock().await;
            job.status = "running".to_string();
        }

        // Find address in background
        let cancel_flag_clone = cancel_flag.clone();
        let pattern_clone2 = pattern_clone.clone();
        let position_clone2 = position_clone.clone();

        let result = tokio::task::spawn_blocking(move || {
            find_vanity_address(&pattern_clone2, &position_clone2, cancel_flag_clone)
        })
        .await;

        // Update job with result
        let mut job = job_ref.lock().await;
        match result {
            Ok(Ok(keypair)) => {
                job.status = "complete".to_string();
                job.result = Some(AddressResult {
                    public_key: keypair.pubkey().to_string(),
                    private_key: bs58::encode(&keypair.to_bytes()).into_string(),
                });
            }
            Ok(Err(err)) => {
                job.status = "error".to_string();
                job.error = Some(err);
            }
            Err(_) => {
                job.status = "error".to_string();
                job.error = Some("Task was canceled".to_string());
            }
        }
    });

    HttpResponse::Ok().json(GenerateResponse { job_id })
}

// Function to find a vanity address
fn find_vanity_address(
    pattern: &str,
    position: &str,
    cancel_flag: Arc<AtomicBool>,
) -> Result<Keypair, String> {
    let pattern_lower = pattern.to_lowercase();

    // Generate keypairs in parallel
    let num_cpus = num_cpus::get();
    let result = (0..num_cpus).into_par_iter().find_map_any(|_| {
        while !cancel_flag.load(Ordering::Relaxed) {
            let keypair = Keypair::new();
            let pubkey_str = keypair.pubkey().to_string();

            let match_found = match position {
                "prefix" => pubkey_str.to_lowercase().starts_with(&pattern_lower),
                "suffix" => pubkey_str.to_lowercase().ends_with(&pattern_lower),
                _ => false,
            };

            if match_found {
                return Some(keypair);
            }
        }
        None
    });

    match result {
        Some(keypair) => Ok(keypair),
        None => Err("Operation was canceled".to_string()),
    }
}

// Get job status
async fn get_status(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let job_id = path.into_inner();

    match data.jobs.get(&job_id) {
        Some(job_ref) => {
            let job = job_ref.lock().await;
            let response = StatusResponse {
                status: job.status.clone(),
                progress: None, // We could compute this if we tracked iterations
                result: job.result.clone(),
                error: job.error.clone(),
            };
            HttpResponse::Ok().json(response)
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Job not found"
        })),
    }
}

// Cancel a job
async fn cancel_job(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let job_id = path.into_inner();

    match data.jobs.get(&job_id) {
        Some(job_ref) => {
            let job = job_ref.lock().await;
            job.cancel_flag.store(true, Ordering::Relaxed);

            HttpResponse::Ok().json(serde_json::json!({
                "status": "cancellation_requested"
            }))
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Job not found"
        })),
    }
}

// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    })
}

// Cleanup task to remove old jobs
async fn cleanup_old_jobs(data: web::Data<AppState>) {
    loop {
        sleep(Duration::from_secs(3600)).await; // Run every hour

        let now = Instant::now();
        let to_remove: Vec<String> = data
            .jobs
            .iter()
            .filter_map(|entry| {
                let job = entry.value().blocking_lock();
                if now.duration_since(job.start_time).as_secs() > 86400 {
                    // Remove jobs older than 24 hours
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for job_id in to_remove {
            data.jobs.remove(&job_id);
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize state
    let app_state = web::Data::new(AppState {
        jobs: DashMap::new(),
    });

    // Start cleanup task
    let state_for_cleanup = app_state.clone();
    tokio::spawn(async move {
        cleanup_old_jobs(state_for_cleanup).await;
    });

    println!("\n");
    println!(" ███████╗ ██████╗ ██╗      █████╗ ███╗   ██╗ █████╗     ██╗   ██╗ █████╗ ███╗   ██╗██╗████████╗██╗   ██╗");
    println!(" ██╔════╝██╔═══██╗██║     ██╔══██╗████╗  ██║██╔══██╗    ██║   ██║██╔══██╗████╗  ██║██║╚══██╔══╝╚██╗ ██╔╝");
    println!(" ███████╗██║   ██║██║     ███████║██╔██╗ ██║███████║    ██║   ██║███████║██╔██╗ ██║██║   ██║    ╚████╔╝ ");
    println!(" ╚════██║██║   ██║██║     ██╔══██║██║╚██╗██║██╔══██║    ╚██╗ ██╔╝██╔══██║██║╚██╗██║██║   ██║     ╚██╔╝  ");
    println!(" ███████║╚██████╔╝███████╗██║  ██║██║ ╚████║██║  ██║     ╚████╔╝ ██║  ██║██║ ╚████║██║   ██║      ██║   ");
    println!(" ╚══════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝      ╚═══╝  ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝   ╚═╝      ╚═╝   ");
    println!("    ");
    println!("                                Built by Ban Github: @ohbanned                                    ");
    println!();

    // Bind server
    let server_address = "127.0.0.1:3001";
    println!("🚀 Server starting on: http://{}", server_address);
    println!();

    // Print new CLI usage instructions
    println!("📋 QUICK USAGE GUIDE:");
    println!("  1. Keep this server running in this terminal");
    println!("  2. Open a new terminal and run one of these commands:");
    println!();
    println!(
        "     \x1b[1;32m./run_cli.sh abc prefix\x1b[0m    # Generate address starting with 'abc'"
    );
    println!(
        "     \x1b[1;32m./run_cli.sh xyz suffix\x1b[0m    # Generate address ending with 'xyz'"
    );
    println!();
    println!("     Replace 'abc' or 'xyz' with your desired 3-8 character pattern");
    println!();
    println!("  3. That's it! Your address will be generated and displayed");
    println!();

    println!("🔍 Press Ctrl+C to stop the server");
    println!();

    // Print API usage for advanced users
    println!("💡 Advanced API Usage:");
    println!("  Generate a vanity address:");
    println!("    curl -X POST http://{}/generate -H \"Content-Type: application/json\" -d '{{\"pattern\":\"abc\",\"position\":\"prefix\"}}'", server_address);
    println!("    {{\"job_id\":\"123e4567-e89b-12d3-a456-426614174000\"}}");
    println!();
    println!("  Check status using the job_id:");
    println!(
        "    curl http://{}/status/123e4567-e89b-12d3-a456-426614174000",
        server_address
    );
    println!("    {{\"status\":\"complete\",\"result\":{{\"public_key\":\"abc...\",\"private_key\":\"...\"}}}}", );
    println!();

    // Start server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/generate", web::post().to(generate_address))
            .route("/status/{job_id}", web::get().to(get_status))
            .route("/cancel/{job_id}", web::post().to(cancel_job))
            .route("/health", web::get().to(health_check))
    })
    .bind(server_address)?
    .run()
    .await
}
