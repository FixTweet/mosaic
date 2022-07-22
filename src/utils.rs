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

use cfg_if::cfg_if;
use image::{EncodableLayout, ImageEncoder, RgbImage};
use image::codecs::png::PngEncoder;
use worker::{Error, Fetch, Headers, Request, RequestInit, Response};

const FAKE_CHROME_VERSION: u16 = 103;

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn image_response(img: RgbImage) -> Result<Response, Error> {
    let mut out = vec![];
    let enc = PngEncoder::new(&mut out);
    enc.write_image(
        img.as_bytes(),
        img.width(),
        img.height(),
        image::ColorType::Rgb8,
    ).expect("failed to encode image");

    Response::from_bytes(out.to_vec())
        .map(|mut res| {
            res.headers_mut()
                .set("Content-Type", "image/png")
                .expect("failed to set headers");
            res
        })
}

pub async fn fetch_image(id: &String) -> Result<RgbImage, Error> {
    let url = format!("https://pbs.twimg.com/media/{}?format=png&name=large", id);
    let mut req = RequestInit::new();

    let mut headers = Headers::new();
    headers.append("sec-ch-ua", &*format!("\".Not/A)Brand\";v=\"99\", \"Google Chrome\";v=\"{version}\", \"Chromium\";v=\"{version}\"", version = FAKE_CHROME_VERSION))?;
    headers.append("DNT", "1")?;
    headers.append("x-twitter-client-language", "en")?;
    headers.append("sec-ch-ua-mobile", "?0")?;
    headers.append("content-type", "application/x-www-form-urlencoded")?;
    headers.append("User-Agent", &*format!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36", FAKE_CHROME_VERSION))?;
    headers.append("x-twitter-active-user", "yes")?;
    headers.append("sec-ch-ua-platform", "\"Windows\"")?;
    headers.append("Accept", "*/*")?;
    headers.append("Origin", "https://twitter.com")?;
    headers.append("Sec-Fetch-Site", "same-site")?;
    headers.append("Sec-Fetch-Mode", "cors")?;
    headers.append("Sec-Fetch-Dest", "empty")?;
    headers.append("Referer", "https://twitter.com/")?;
    headers.append("Accept-Encoding", "gzip, deflate, br")?;
    headers.append("Accept-Language", "en")?;
    req.with_headers(headers);

    let mut res = Fetch::Request(Request::new_with_init(&*url, &req)?).send().await?;
    let img = image::load_from_memory(&*res.bytes().await?);
    return match img {
        Ok(img) => Ok(img.into_rgb8()),
        Err(_) => Err(Error::from("Invalid image"))
    }
}
