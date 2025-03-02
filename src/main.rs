use std::sync::Arc;
use actix_web::{post, get, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use actix_web::http::header;
use serde::{Serialize, Deserialize};
use solana_sdk::signature::{Keypair, Signer};
use uuid::Uuid;
use dashmap::DashMap;
use bs58;
use log::{info, debug};
use env_logger;
use chrono;

#[derive(Debug, Deserialize)]
struct VanityRequest {
    pattern: String,
    position: String,
}

#[derive(Debug, Serialize)]
struct VanityResponse {
    public_key: String,
    private_key: String,
}

#[derive(Debug, Serialize)]
struct JobResponse {
    job_id: String,
}

struct AppState {
    jobs: Arc<DashMap<String, bool>>,         // job_id -> cancelled flag
    results: Arc<DashMap<String, VanityResponse>>,
}

#[post("/generate")]
async fn generate(
    req: web::Json<VanityRequest>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let pattern = req.pattern.to_lowercase();
    if pattern.len() < 3 || pattern.len() > 8 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Pattern must be between 3 and 8 characters"
        }));
    }

    let job_id = Uuid::new_v4().to_string();
    data.jobs.insert(job_id.clone(), false);

    let jobs = data.jobs.clone();
    let results = data.results.clone();
    let job_id_clone = job_id.clone();
    let position = req.position.clone();
    let pattern_clone = pattern.clone();

    // Spawn the generation task
    tokio::spawn(async move {
        debug!("Starting generation for pattern: {}", pattern_clone);
        
        loop {
            // Check if cancelled
            if let Some(cancelled) = jobs.get(&job_id_clone) {
                if *cancelled {
                    debug!("Job {} cancelled", job_id_clone);
                    break;
                }
            } else {
                debug!("Job {} not found", job_id_clone);
                break;
            }

            let keypair = Keypair::new();
            let address = keypair.pubkey().to_string().to_lowercase();
            
            let matches = match position.as_str() {
                "prefix" => address.starts_with(&pattern_clone),
                "suffix" => address.ends_with(&pattern_clone),
                _ => false,
            };

            if matches {
                info!("Found matching address: {}", address);
                results.insert(job_id_clone.clone(), VanityResponse {
                    public_key: keypair.pubkey().to_string(),
                    private_key: bs58::encode(keypair.to_bytes()).into_string(),
                });
                break;
            }
        }
    });

    HttpResponse::Ok().json(JobResponse { job_id })
}

#[get("/status/{job_id}")]
async fn status(
    job_id: web::Path<String>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let job_id = job_id.into_inner();
    
    if let Some(result) = data.results.get(&job_id) {
        // Found a result
        let result = result.value();
        HttpResponse::Ok().json(serde_json::json!({
            "status": "complete",
            "result": result
        }))
    } else if data.jobs.contains_key(&job_id) {
        // Still running
        HttpResponse::Ok().json(serde_json::json!({
            "status": "running"
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Job not found"
        }))
    }
}

#[post("/cancel/{job_id}")]
async fn cancel(
    job_id: web::Path<String>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let job_id = job_id.into_inner();
    
    if let Some(mut job) = data.jobs.get_mut(&job_id) {
        *job = true;  // Set cancelled flag
        HttpResponse::Ok().json(serde_json::json!({
            "status": "cancelled"
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Job not found"
        }))
    }
}

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting vanity address generator server...");
    
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let bind_addr = format!("{}:{}", host, port);
    
    let app_state = web::Data::new(AppState {
        jobs: Arc::new(DashMap::new()),
        results: Arc::new(DashMap::new()),
    });
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .max_age(3600);
            
        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(generate)
            .service(status)
            .service(cancel)
            .service(health)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
