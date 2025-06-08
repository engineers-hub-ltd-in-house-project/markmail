// CORS設定
// TODO: 本番環境用のCORS設定

use axum::http::{header, HeaderValue, Method};
use tower_http::cors::CorsLayer;

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_credentials(true)
}
