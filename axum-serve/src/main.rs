use std::{net::SocketAddr, str::FromStr};

use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use http::{HeaderName, HeaderValue};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, set_header::SetResponseHeaderLayer};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/", ServeDir::new("../dist"))
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
