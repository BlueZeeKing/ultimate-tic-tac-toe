use std::{net::SocketAddr, str::FromStr, sync::Arc};

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dashmap::DashMap;
use http::{HeaderName, HeaderValue};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, set_header::SetResponseHeaderLayer};
use ultimate_tic_tac_toe::{Board, IndividualBoard, LocalBoardState, MiniMaxResult, Player};

#[derive(Clone)]
struct Cache {
    eval_cache: DashMap<(IndividualBoard, Player), f64>,
    eval_cache2: DashMap<([Option<LocalBoardState>; 9], Player), f64>,
}

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
        )
        .with_state(Arc::new(Cache {
            eval_cache: Default::default(),
            eval_cache2: Default::default(),
        }));

    axum_server::bind(SocketAddr::from_str("0.0.0.0:3000").unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn calc(State(state): State<Arc<Cache>>, Json(board): Json<Board>) -> impl IntoResponse {
    let ((global, local), eval, _) =
        board.minimax(6, &DashMap::new(), &state.eval_cache, &state.eval_cache2);

    Json(MiniMaxResult {
        global,
        local,
        eval,
    })
}
