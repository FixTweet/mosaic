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
use std::time::{Duration, Instant};

use warp::http::Response;
use warp::{path, Filter, Reply};

use crate::mosaic::mosaic;
use crate::utils::{fetch_image, image_response, ImageType};

mod mosaic;
mod utils;

async fn handle(
    image_type: ImageType,
    _id: String,
    image_ids: Vec<String>,
) -> Response<warp::hyper::Body> {
    let client = reqwest::ClientBuilder::default()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let images: VecDeque<_> = futures::future::join_all(
        image_ids
            .iter()
            .map(|image_id| fetch_image(&client, image_id)),
    )
    .await
    .into_iter()
    .filter_map(std::convert::identity)
    .collect();

    if images.is_empty() {
        return Response::builder()
            .status(500)
            .body("Failed to download all images.")
            .unwrap()
            .into_response();
    }

    let start = Instant::now();
    let image = match tokio::task::spawn_blocking(move || mosaic(images)).await {
        Ok(image) => image,
        Err(_err) => {
            return Response::builder()
                .status(500)
                .body("Failed to await compose image task.")
                .unwrap()
                .into_response();
        }
    };
    let size = format!("{0}x{1}", image.width(), image.height());
    let mosaic_time = start.elapsed();

    let encoding_start = Instant::now();
    let encoded = match image_response(image, image_type) {
        Ok(res) => res.into_response(),
        Err(_) => {
            return Response::builder()
                .status(500)
                .body("Failed to encode image")
                .unwrap()
                .into_response()
        }
    };

    println!(
        "Took {time}ms (mosaic: {mosaic}ms, encoding: {enc}ms) to process: {ids}. Image size: {size}.",
        time = start.elapsed().as_millis(),
        mosaic = mosaic_time.as_millis(),
        ids = image_ids.join(", "),
        enc = encoding_start.elapsed().as_millis(),
        size = size,
    );
    encoded
}

#[tokio::main]
async fn main() {
    let routes = warp::get().and(
        path!(ImageType / String / String / String / String / String)
            .then(|image_type, id, a, b, c, d| handle(image_type, id, vec![a, b, c, d]))
            .or(path!(ImageType / String / String / String / String)
                .then(|image_type, id, a, b, c| handle(image_type, id, vec![a, b, c])))
            .or(path!(ImageType / String / String / String)
                .then(|image_type, id, a, b| handle(image_type, id, vec![a, b]))),
    );

    let port = std::env::var("PORT")
        .ok()
        .unwrap_or_else(|| "3030".to_string())
        .parse()
        .expect("PORT environment variable is not an u16.");

    println!("Starting fixtweet-mosaic on on 127.0.0.1:{}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
