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

use itertools::Itertools;
use worker::*;

use crate::mosaic::mosaic;
use crate::utils::{fetch_image, image_response};

mod utils;
mod mosaic;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/", |req, _ctx| async move {
            let url = req.url().unwrap();
            let image_ids = url.query_pairs()
                .sorted_by(|(a, _), (b, _)| a.cmp(b))
                .map(|(_, id)| id.to_string())
                .collect_vec();

            if image_ids.len() == 0 {
                return Response::error("Bad request", 400);
            }

            let images = futures::future::join_all(image_ids.iter().map(fetch_image)).await
                .into_iter()
                .filter(|i| {
                    if !i.is_ok() {
                        console_error!("Failed to download image");
                        return false;
                    }
                    return true;
                }).map(|i| i.unwrap())
                .collect_vec();

            if images.len() == 0 {
                return Response::error("Failed to download all images.", 500);
            }

            let result = mosaic(VecDeque::from(images));
            if result.is_none() {
                return Response::error("Too many images supplied", 400);
            }

            image_response(result.unwrap())
        })
        .run(req, env)
        .await
}
