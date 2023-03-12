use axum::{
    extract::{
        ws::{WebSocketUpgrade},
        TypedHeader,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use std::{net::SocketAddr, env};
use tower_http::{
    trace::{DefaultMakeSpan, TraceLayer}, cors::CorsLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::extract::connect_info::ConnectInfo;

mod audio;
mod track;
mod dal;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    let database_connection = sea_orm::Database::connect(db_url)
        .await
        .expect("Database connection failed");
    
    let cors = CorsLayer::new()
        .allow_methods(tower_http::cors::Any {})
        .allow_headers(tower_http::cors::Any {})
        .allow_origin(tower_http::cors::Any {});
    
    let state = AppState { database_connection };

    let app = Router::new()
        .route("/ws", get(establish_ws_connection))
        .route("/tracks", get(track::restore).post(track::store))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct AppState {
    database_connection: sea_orm::DatabaseConnection,
}

async fn establish_ws_connection(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| audio::streaming::handle_connection(socket, addr))
}