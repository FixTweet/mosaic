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

use std::str::FromStr;

use image::{EncodableLayout, ImageEncoder, ImageError, RgbImage};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use reqwest::Client;
use reqwest::header::HeaderMap;
use warp::http::Response;

const FAKE_CHROME_VERSION: u16 = 103;

pub enum ImageType {
    WebP,
    PNG,
    JPEG,
}

impl FromStr for ImageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "webp" => Ok(ImageType::WebP),
            "png" => Ok(ImageType::PNG),
            "jpeg" => Ok(ImageType::JPEG),
            _ => Err(())
        }
    }
}

pub fn image_response(img: RgbImage, encoder: ImageType) -> Result<Response<Vec<u8>>, ImageError> {
    let encoded = match encoder {
        ImageType::WebP => webp::Encoder::from_rgb(
            img.as_bytes(),
            img.width(),
            img.height(),
        ).encode(90.0).to_vec(),

        ImageType::PNG => {
            let mut out = vec![];
            let enc = PngEncoder::new(&mut out);
            enc.write_image(
                img.as_bytes(),
                img.width(),
                img.height(),
                image::ColorType::Rgb8,
            )?;
            out.to_vec()
        }

        ImageType::JPEG => {
            let mut out = vec![];
            let enc = JpegEncoder::new(&mut out);
            enc.write_image(
                img.as_bytes(),
                img.width(),
                img.height(),
                image::ColorType::Rgb8,
            )?;
            out.to_vec()
        }
    };

    let content_type = match encoder {
        ImageType::WebP => "image/webp",
        ImageType::PNG => "image/png",
        ImageType::JPEG => "image/jpeg"
    };

    Ok(
        Response::builder()
            .status(200)
            .header("Content-Type", content_type)
            .body(encoded)
            .unwrap()
    )
}

pub async fn fetch_image(id: &String) -> Option<RgbImage> {
    // TODO keep this in memory
    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.append("sec-ch-ua", format!("\".Not/A)Brand\";v=\"99\", \"Google Chrome\";v=\"{version}\", \"Chromium\";v=\"{version}\"", version = FAKE_CHROME_VERSION).parse().unwrap());
    headers.append("DNT", "1".parse().unwrap());
    headers.append("x-twitter-client-language", "en".parse().unwrap());
    headers.append("sec-ch-ua-mobile", "?0".parse().unwrap());
    headers.append("content-type", "application/x-www-form-urlencoded".parse().unwrap());
    headers.append("User-Agent", format!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36", FAKE_CHROME_VERSION).parse().unwrap());
    headers.append("x-twitter-active-user", "yes".parse().unwrap());
    headers.append("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
    headers.append("Accept", "*/*".parse().unwrap());
    headers.append("Origin", "https://twitter.com".parse().unwrap());
    headers.append("Sec-Fetch-Site", "same-site".parse().unwrap());
    headers.append("Sec-Fetch-Mode", "cors".parse().unwrap());
    headers.append("Sec-Fetch-Dest", "empty".parse().unwrap());
    headers.append("Referer", "https://twitter.com/".parse().unwrap());
    headers.append("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
    headers.append("Accept-Language", "en".parse().unwrap());

    let res = client
        .get(format!("https://pbs.twimg.com/media/{}?format=png&name=large", id))
        .headers(headers)
        .send()
        .await.ok()?;
    let img = image::load_from_memory(&*res.bytes().await.ok()?);
    return match img {
        Ok(img) => Some(img.into_rgb8()),
        Err(_) => None
    }
}
