use std::{net::SocketAddr, str::FromStr, sync::Arc};

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dashmap::DashMap;
use http::{HeaderName, HeaderValue};
use minimax::minimax;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, set_header::SetResponseHeaderLayer};
use ultimate_tic_tac_toe::{Board, IndividualBoard, LocalBoardState, MiniMaxResult, Player};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/calc", post(calc))
        .nest_service("/", ServeDir::new("../client/dist"))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::new())
                .layer(SetResponseHeaderLayer::overriding(
                    HeaderName::from_static("cross-origin-opener-policy"),
                    HeaderValue::from_static("same-origin"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    HeaderName::from_static("cross-origin-embedder-policy"),
                    HeaderValue::from_static("credentialless"),
                )),
        );

    axum_server::bind(SocketAddr::from_str("0.0.0.0:3000").unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn calc(Json(board): Json<Board>) -> impl IntoResponse {
    let ((global, local), eval, _) = minimax(&board, 10, 2, f64::MIN, f64::MAX);

    Json(MiniMaxResult {
        global,
        local,
        eval,
    })
}
