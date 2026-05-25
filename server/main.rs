use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
struct VerifyRequest {
    file_url: String,  // S3/HTTP URL or base64
}

#[derive(Serialize)]
struct VerifyResponse {
    valid: bool,
    metadata: Option<AIMFMetadata>,
    error: Option<String>,
}

async fn verify_handler(req: VerifyRequest) -> Result<impl warp::Reply, warp::Rejection> {
    // Download or decode file
    let bytes = download_file(&req.file_url).await?;
    
    // Verify AIMF
    match aimf::verify(&bytes) {
        Ok(metadata) => Ok(warp::reply::json(&VerifyResponse {
            valid: true,
            metadata: Some(metadata),
            error: None,
        })),
        Err(e) => Ok(warp::reply::json(&VerifyResponse {
            valid: false,
            metadata: None,
            error: Some(e.to_string()),
        })),
    }
}

#[tokio::main]
async fn main() {
    let verify_route = warp::post()
        .and(warp::path("verify"))
        .and(warp::body::json())
        .and_then(verify_handler);
    
    // Health check for k8s
    let health_route = warp::get()
        .and(warp::path("health"))
        .map(|| "OK");
    
    let routes = verify_route.or(health_route);
    
    println!("🚀 AIMF Server running on port 8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}