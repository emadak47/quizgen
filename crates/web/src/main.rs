mod error;
mod routes;
mod session;

use askama::Template;
use askama_web::WebTemplate;
use axum::{routing::get, routing::post, Router};
use clap::Parser;
use session::SessionStore;
use std::path::PathBuf;
use tower_cookies::CookieManagerLayer;

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
}

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn index() -> IndexTemplate {
    IndexTemplate
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Validate source directory exists and contains .txt files
    if !args.source.is_dir() {
        anyhow::bail!("Source directory does not exist: {:?}", args.source);
    }
    let has_txt = std::fs::read_dir(&args.source)?
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("txt"));
    if !has_txt {
        anyhow::bail!("No .txt files found in source directory: {:?}", args.source);
    }

    // Validate API keys are present
    for key in ["WORDS_API_KEY", "COLLEGIATE_API_KEY", "THESAURUS_API_KEY"] {
        if std::env::var(key).is_err() {
            anyhow::bail!("Missing environment variable: {key}");
        }
    }

    let state = AppState {
        store: SessionStore::new(),
        source_dir: args.source.clone(),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/quiz/start", post(routes::start_quiz))
        .route("/quiz/question/{n}", get(routes::show_question))
        .route("/quiz/answer/{n}", post(routes::submit_answer))
        .route("/quiz/results", get(routes::show_results))
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
