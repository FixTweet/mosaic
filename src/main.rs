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

use crate::mosaic::mosaic;
use crate::utils::{fetch_image, image_response};

mod mosaic;
mod utils;

#[derive(Deserialize)]
struct HandlePath {
    image_type: ImageType,
    image_ids: String,
}

#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageType {
    Webp,
    Png,
    Jpeg,
}

async fn handle_axum(
    path: Path<HandlePath>,
    Extension(client): Extension<reqwest::Client>,
) -> impl IntoResponse {
    let image_ids: Vec<_> = path
        .image_ids
        .split('/')
        .filter(|image_id| !image_id.is_empty())
        .collect();

    let start = Instant::now();
    let images: VecDeque<_> = futures::future::join_all(
        image_ids
            .iter()
            .map(|image_id| fetch_image(&client, image_id)),
    )
    .await
    .into_iter()
    .filter_map(std::convert::identity)
    .collect();
    let download_time = start.elapsed();

    if images.is_empty() {
        return (StatusCode::BAD_REQUEST, "No images could be downloaded").into_response();
    }

    let mosaic_start = Instant::now();
    let image = match tokio::task::spawn_blocking(move || mosaic(images)).await {
        Ok(image) => image,
        Err(_err) => {
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
        Err(_err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Image could not be encoded.",
            )
                .into_response();
        }
    };

    println!(
        "Took {time}ms (download: {download}ms, mosaic: {mosaic}ms, encoding: {enc}ms) to process: {ids}. Image size: {size}.",
        time = start.elapsed().as_millis(),
        download = download_time.as_millis(),
        mosaic = mosaic_time.as_millis(),
        ids = image_ids.join(", "),
        enc = encoding_start.elapsed().as_millis(),
        size = size,
    );

    encoded
}

#[tokio::main]
async fn main() {
    let client = reqwest::ClientBuilder::default()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let app = Router::new()
        .route("/:image_type/:tweet_id/*image_ids", get(handle_axum))
        .layer(Extension(client));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_err| "3030".to_string())
        .parse()
        .expect("PORT was invalid");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Starting fixtweet-mosaic on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
