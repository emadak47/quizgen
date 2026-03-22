mod error;
mod routes;
mod session;

use std::path::PathBuf;

use axum::{routing::get, routing::post, Router};
use tower_http::services::ServeDir;
use clap::Parser;
use session::SessionStore;
use tower_cookies::CookieManagerLayer;

const STATIC_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/static");
const WORDS_API_KEY: &str = "WORDS_API_KEY";
const COLLEGIATE_API_KEY: &str = "COLLEGIATE_API_KEY";
const THESAURUS_API_KEY: &str = "THESAURUS_API_KEY";

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "source", env = "SOURCE_DIR")]
    source: PathBuf,
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[derive(Clone)]
pub struct AppState {
    pub store: SessionStore,
    pub source_dir: PathBuf,
    pub words_api_key: String,
    pub collegiate_key: String,
    pub thesaurus_key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    if !args.source.is_dir() {
        anyhow::bail!("Source directory does not exist: {:?}", args.source);
    }
    let has_txt = std::fs::read_dir(&args.source)?
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("txt"));
    if !has_txt {
        anyhow::bail!("No .txt files found in source directory: {:?}", args.source);
    }

    let state = AppState {
        store: SessionStore::new(),
        source_dir: args.source,
        words_api_key: std::env::var(WORDS_API_KEY)?,
        collegiate_key: std::env::var(COLLEGIATE_API_KEY)?,
        thesaurus_key: std::env::var(THESAURUS_API_KEY)?,
    };

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/quiz/start", post(routes::start_quiz))
        .route("/quiz/question/{n}", get(routes::show_question))
        .route("/quiz/answer/{n}", post(routes::submit_answer))
        .route("/quiz/results", get(routes::show_results))
        .nest_service("/static", ServeDir::new(STATIC_DIR))
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
