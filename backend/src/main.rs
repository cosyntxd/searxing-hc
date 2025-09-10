pub mod data;
pub mod database;
pub mod embedder;
pub mod links;

use axum::http::StatusCode;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    response::IntoResponse,
    routing::{get, get_service, post},
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    process::exit,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{signal, time};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{data::ScrapedMainPageEnum, database::Database};

struct AppState {
    data: Database,
    secret: String,
    start_time: Instant,
}

async fn periodic_saves(state: Arc<AppState>) {
    let mut interval = time::interval(Duration::from_secs(15));
    interval.tick().await;
    loop {
        interval.tick().await;
        println!("saving db...");
        state.data.save_json();
        println!("completed saving db");
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct AddDataRequest {
    secret: String,
    data: String,
}

async fn add_data(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<AddDataRequest>,
) -> impl IntoResponse {
    if payload.secret != app_state.secret {
        return (StatusCode::UNAUTHORIZED, "Invalid secret".to_string()).into_response();
    }

    let entry: ScrapedMainPageEnum = match serde_json::from_str(&payload.data) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to parse data as JSON: {}", e);
            return (StatusCode::BAD_REQUEST, format!("Invalid JSON data: {}", e)).into_response();
        }
    };

    app_state.data.add_entry(entry);

    let response_message = format!("Approx size of db: {}", -1);

    (StatusCode::OK, response_message).into_response()
}
#[derive(Deserialize, Serialize, Debug)]
struct SearchInputRequest {
    q: String,
}
async fn query_sort(
    State(app_state): State<Arc<AppState>>,
    Query(payload): Query<SearchInputRequest>,
) -> impl IntoResponse {
    let db_load_start = Instant::now();
    let search_results = app_state.data.search_and_rank_json(payload.q, 500);
    println!("sort took: {:?}", db_load_start.elapsed());
    (StatusCode::OK, search_results).into_response()
}

#[derive(Deserialize, Debug)]
struct GetPreviewRequest {
    uuid: usize,
}
async fn get_preview(
    State(app_state): State<Arc<AppState>>,
    Query(payload): Query<GetPreviewRequest>,
) -> impl IntoResponse {
    // let db_load_start = Instant::now();
    // let data_guard = app_state.data.raw_data.read().unwrap();
    // if payload.uuid >= data_guard.length {
    //     return (StatusCode::NOT_FOUND, "UUID not found".to_string()).into_response();
    // }
    // let page = &data_guard.raw_text[payload.uuid].clone();
    // drop(data_guard);

    // match serde_json::to_string(page) {
    //     Ok(json) => {
    //         println!("loaded preview in: {:?}", db_load_start.elapsed());
    //         (StatusCode::OK, json).into_response()
    //     }
    //     Err(e) => {
    //         eprintln!("Failed to serialize page preview: {}", e);
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             "Failed to serialize preview".to_string(),
    //         )
    //             .into_response()
    //     }
    // }
}

#[derive(Deserialize, Serialize, Debug)]
struct SetExtrasRequest {
    secret: String,
    id: usize,
    score_multiplier: Option<f32>,
    embedding: Option<Vec<f32>>,
}
async fn set_extras(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<SetExtrasRequest>,
) -> impl IntoResponse {
    if payload.secret != app_state.secret {
        return (StatusCode::UNAUTHORIZED, "Invalid secret".to_string()).into_response();
    }

    // if payload.id >= app_state.data.raw_data.read().unwrap().length {
    //     return (StatusCode::NOT_FOUND, "ID not found".to_string()).into_response();
    // }

    todo!()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let database = Database::load_file("../complete_database.json");

    let secret = env::var("SOM_BACKEND_AUTH_SECRET").unwrap_or_else(|_| {
        eprintln!("no secret is an oops in prod, using default.");
        "not_a_secret_secret".into()
    });
    let state = Arc::new(AppState {
        data: database,
        secret,
        start_time: Instant::now(),
    });

    let cors_layer = CorsLayer::new()
        .allow_origin(["http://localhost:6552".parse::<HeaderValue>().unwrap()])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT]);

    let app = Router::new()
        .route(
            "/",
            get_service(ServeDir::new(
                "/Users/ryan/Github/Hackclub-projects/website/index.html",
            )),
        )
        .route("/add", post(add_data))
        .route("/query", get(query_sort))
        .route("/preview", get(get_preview))
        .route("/set_extras", post(set_extras))
        .with_state(Arc::clone(&state))
        .layer(cors_layer);

    // tokio::spawn(periodic_saves(Arc::clone(&state)));

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("failed to listen for ctrl_c");
        println!("\ntrying to close, saving state...");
        state.data.save_json();
        exit(0)
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:6552")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
    // let path = PathBuf::from("/home/searxing/.searxing.hackclub.app.axum.sock");

    // let _ = tokio::fs::remove_file(&path).await;
    // tokio::fs::create_dir_all(path.parent().unwrap())
    //     .await
    //     .unwrap();

    // let uds = UnixListener::bind(path.clone()).unwrap();
    // tokio::spawn(async move {
    //     axum::serve(uds, app).await.unwrap();
    // });
}
