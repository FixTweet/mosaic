/*
 * MIT License
 *
 * Copyright (c) 2022 Antonio32A (antonio32a.com) <~@antonio32a.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Router,
};
use serde::Deserialize;
use tracing::instrument;

use crate::mosaic::mosaic;
use crate::utils::{fetch_image, image_response};

mod mosaic;
mod utils;

#[derive(Debug, Deserialize)]
struct HandlePath {
    image_type: ImageType,
    image_ids: String,
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageType {
    Webp,
    Png,
    Jpeg,
}

#[instrument(skip(path, client))]
async fn handle(
    path: Path<HandlePath>,
    Extension(client): Extension<reqwest::Client>,
) -> impl IntoResponse {
    let image_ids: Vec<_> = path
        .image_ids
        .split('/')
        .filter(|image_id| !image_id.is_empty())
        .collect();

    tracing::info!(image_type = ?path.image_type, "given image ids: {}", image_ids.join(", "));

    let start = Instant::now();
    let images: VecDeque<_> = futures::future::join_all(
        image_ids
            .iter()
            .map(|image_id| fetch_image(&client, image_id)),
    )
    .await
    .into_iter()
    .flatten()
    .collect();
    let download_time = start.elapsed();

    if images.is_empty() {
        tracing::warn!("no images were found");
        return (StatusCode::BAD_REQUEST, "No images could be found.").into_response();
    }

    let mosaic_start = Instant::now();
    let image = match tokio::task::spawn_blocking(move || mosaic(images)).await {
        Ok(image) => image,
        Err(err) => {
            tracing::error!("could not spawn mosaic task: {}", err);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Mosaic task failed to complete.",
            )
                .into_response();
        }
    };
    let mosaic_time = mosaic_start.elapsed();
    let size = format!("{0}x{1}", image.width(), image.height());

    let encoding_start = Instant::now();
    let encoded = match image_response(image, path.image_type) {
        Ok(res) => res.into_response(),
        Err(err) => {
            tracing::error!("could not encode image: {}", err);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Image could not be encoded.",
            )
                .into_response();
        }
    };

    tracing::info!(
        time = start.elapsed().as_millis(),
        download = download_time.as_millis(),
        mosaic = mosaic_time.as_millis(),
        encoding = encoding_start.elapsed().as_millis(),
        "completed encode with final dimensions: {}",
        size
    );

    encoded
}

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let client = reqwest::ClientBuilder::default()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let app = Router::new()
        .route("/:image_type/:tweet_id/*image_ids", get(handle))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(Extension(client));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_err| "3030".to_string())
        .parse()
        .expect("PORT was invalid");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!("starting fixtweet-mosaic on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
