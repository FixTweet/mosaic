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
use std::time::Instant;

use warp::{Filter, path, Reply};
use warp::http::Response;
use warp::hyper::Body;

use crate::mosaic::{mosaic, mosaic_size};
use crate::utils::{fetch_images, image_response, ImageType};

mod utils;
mod mosaic;

async fn handle(image_type: ImageType, _id: String, image_ids: Vec<String>) -> Response<Body> {
    let images = fetch_images(&image_ids).await;
    if images.len() == 0 {
        return Response::builder()
            .status(500)
            .body("Failed to download all images.")
            .unwrap()
            .into_response();
    }

    let start = Instant::now();
    let image = mosaic(VecDeque::from(images));
    let size = format!("{0}x{1}", image.width(), image.height());
    let mosaic_time = start.elapsed();

    let encoding_start = Instant::now();
    let encoded = match image_response(image, image_type) {
        Ok(res) => res.into_response(),
        Err(_) => return Response::builder()
            .status(500)
            .body("Failed to encode image")
            .unwrap()
            .into_response()
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

async fn handle_size(_id: String, image_ids: Vec<String>) -> Response<Body> {
    let images = fetch_images(&image_ids).await;
    if images.len() == 0 {
        return Response::builder()
            .status(500)
            .body("Failed to download all images.")
            .unwrap()
            .into_response();
    }

    let size = mosaic_size(VecDeque::from(images));
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&size).unwrap())
        .unwrap()
        .into_response()
}

#[tokio::main]
async fn main() {
    let routes = warp::get().and(
        path!(ImageType / String / String / String / String / String)
            .then(|image_type, id, a, b, c, d|
                handle(image_type, id, vec![a, b, c, d])
            )
            .or(
                path!(ImageType / String / String / String / String)
                    .then(|image_type, id, a, b, c|
                        handle(image_type, id, vec![a, b, c])
                    )
            )
            .or(
                path!(ImageType / String / String / String)
                    .then(|image_type, id, a, b|
                        handle(image_type, id, vec![a, b])
                    )
            )
            .or(
                path!("size" / String / String / String / String / String)
                    .then(|id, a, b, c, d|
                        handle_size(id, vec![a, b, c, d])
                    )
            )
            .or(
                path!("size" / String / String / String / String)
                    .then(|id, a, b, c|
                        handle_size(id, vec![a, b, c])
                    )
            )
            .or(
                path!("size" / String / String / String)
                    .then(|id, a, b|
                        handle_size(id, vec![a, b])
                    )
            ),
    );

    let port = option_env!("PORT")
        .unwrap_or("3030")
        .parse::<u16>()
        .ok()
        .expect("PORT environment variable is not an u16.");

    println!("Starting pxtwitter-mosaic on on 127.0.0.1:{}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
