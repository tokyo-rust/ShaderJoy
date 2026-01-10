use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::generation::{Specimen, generate_specimens};
use crate::shader_gen::config::ShaderGenConfig;

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub parent: Option<ParentSpecimen>,
    pub word_bank: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ParentSpecimen {
    pub code: String,
    pub prompt_words: Vec<String>,
    pub generation: u32,
}

impl From<ParentSpecimen> for Specimen {
    fn from(p: ParentSpecimen) -> Self {
        Specimen {
            code: p.code,
            prompt_words: p.prompt_words,
            generation: p.generation,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub shaders: Vec<String>,
    pub errors: Vec<String>,
}

async fn generate_handler(Json(req): Json<GenerateRequest>) -> Json<GenerateResponse> {
    let mut config = ShaderGenConfig::default();
    config.word_bank = req.word_bank;

    let parent = req.parent.map(|p| p.into());
    let permutation_cnt = 9;

    let results = generate_specimens(parent, permutation_cnt, &config).await;

    let mut shaders = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(specimen) => shaders.push(specimen.code),
            Err(e) => errors.push(e.to_string()),
        }
    }

    Json(GenerateResponse { shaders, errors })
}

pub fn api_router() -> Router {
    Router::new().route("/api/generate", post(generate_handler))
}

pub async fn run_server(addr: SocketAddr) {
    let app = api_router();

    println!("Starting API server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
