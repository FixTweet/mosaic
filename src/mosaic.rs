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

use std::{collections::VecDeque, time::Instant};

use image::{imageops::FilterType, RgbImage};
use tracing::instrument;

const SPACING_SIZE: u32 = 10;
const MAX_SIZE: u32 = 4000;

pub fn mosaic(mut images: VecDeque<RgbImage>) -> RgbImage {
    match images.len() {
        2 => {
            let first = images.pop_front().unwrap();
            let second = images.pop_front().unwrap();
            build_2_mosaic(first, second);
        }
        3 => {
            let first = images.pop_front().unwrap();
            let second = images.pop_front().unwrap();
            let third = images.pop_front().unwrap();
            build_3_mosaic(first, second, third);
        }
        4 => {
            let first = images.pop_front().unwrap();
            let second = images.pop_front().unwrap();
            let third = images.pop_front().unwrap();
            let fourth = images.pop_front().unwrap();
            mosaic_4(first, second, third, fourth);

        }
        _ => panic!("impossible image length"),
    }
}

fn mosaic_4(first: RgbImage, second: RgbImage, third: RgbImage, fourth: RgbImage) -> RgbImage {
    let top = calc_horizontal_size(&first, &second);
    let bottom = calc_horizontal_size(&third, &fourth);
    let all = calc_vertical_size_raw(
        Size {
            width: top.width,
            height: top.height,
        },
        Size {
            width: bottom.width,
            height: bottom.height,
        },
    );
    let top_width_mult = all.first_height as f32 / top.height as f32;
    let bottom_width_mult = all.second_height as f32 / bottom.height as f32;
    let size_mult = calc_multiplier(Size {
        width: all.width,
        height: all.height,
    });

    let [first, second, third, fourth] = resize_images([
        (
            first,
            Size {
                width: (top.first_width as f32 * top_width_mult * size_mult).round() as u32,
                height: (all.first_height as f32 * size_mult).round() as u32,
            },
        ),
        (
            second,
            Size {
                width: (top.second_width as f32 * top_width_mult * size_mult).round()
                    as u32,
                height: (all.first_height as f32 * size_mult).round() as u32,
            },
        ),
        (
            third,
            Size {
                width: (bottom.first_width as f32 * bottom_width_mult * size_mult).round()
                    as u32,
                height: (all.second_height as f32 * size_mult) as u32,
            },
        ),
        (
            fourth,
            Size {
                width: (bottom.second_width as f32 * bottom_width_mult * size_mult).round()
                    as u32,
                height: (all.second_height as f32 * size_mult) as u32,
            },
        ),
    ]);

    let mut background = create_background(
        (all.width as f32 * size_mult) as u32,
        (all.height as f32 * size_mult) as u32,
    );

    // We also multiply the spacing by how much the width increased, this isn't ideal but
    // it's barely noticeable and it's how the original FixTweet-Mosaic code works.
    image::imageops::overlay(&mut background, &first, 0, 0);
    image::imageops::overlay(
        &mut background,
        &second,
        (first.width() as f32 + SPACING_SIZE as f32 * top_width_mult) as i64,
        0,
    );
    image::imageops::overlay(
        &mut background,
        &third,
        0,
        (first.height() + SPACING_SIZE) as i64,
    );
    image::imageops::overlay(
        &mut background,
        &fourth,
        (third.width() as f32 + SPACING_SIZE as f32 * bottom_width_mult) as i64,
        (first.height() + SPACING_SIZE) as i64,
    );
    background
}

fn create_background(width: u32, height: u32) -> RgbImage {
    RgbImage::from_pixel(width, height, image::Rgb([0, 0, 0]))
}

fn calc_horizontal_size(first: &RgbImage, second: &RgbImage) -> HorizontalSize {
    calc_horizontal_size_raw(
        Size {
            width: first.width(),
            height: first.height(),
        },
        Size {
            width: second.width(),
            height: second.height(),
        },
    )
}

fn calc_horizontal_size_raw(first: Size, second: Size) -> HorizontalSize {
    let mut small = second;
    let mut big = first;
    let mut swapped = false;
    if second.height > first.height {
        small = first;
        big = second;
        swapped = true
    }

    let small_width = (big.height as f32 / small.height as f32 * small.width as f32).round() as u32;
    HorizontalSize {
        width: small_width + SPACING_SIZE + big.width,
        height: big.height,
        first_width: if swapped { small_width } else { big.width },
        second_width: if swapped { big.width } else { small_width },
    }
}

fn calc_vertical_size_raw(first: Size, second: Size) -> VerticalSize {
    let mut small = second;
    let mut big = first;
    let mut swapped = false;
    if second.width > first.width {
        small = first;
        big = second;
        swapped = true
    }

    let small_height = (big.width as f32 / small.width as f32 * small.height as f32).round() as u32;
    VerticalSize {
        width: big.width,
        height: small_height + SPACING_SIZE + big.height,
        first_height: if swapped { small_height } else { big.height },
        second_height: if swapped { big.height } else { small_height },
    }
}

fn calc_multiplier(size: Size) -> f32 {
    let biggest = if size.width > size.height {
        size.width
    } else {
        size.height
    };

    if biggest > MAX_SIZE {
        MAX_SIZE as f32 / biggest as f32
    } else {
        1.0
    }
}


fn scale_height_dimension(image_size: Size, other_height: u32) -> Size {
    let scale_factor = image_size.height as f32 / other_height as f32;
    Size {
        (image_size.width as f32 / scale_factor).round() as u32,
        image_size.height,
    }
}

fn scale_width_dimension(image_size: Size, other_width: u32) -> Size {
    let scale_factor = image_size.width as f32 / other_width as f32;
    Size {
        image_size.width,
        (image_size.height as f32 / scale_factor).round() as u32,
    }
}

fn overall_scale_factor(size: Size) -> f32 {
    let biggest = if size.width > size.height {
        size.width
    } else {
        size.height
    };

    if biggest > MAX_SIZE {
        MAX_SIZE as f32 / biggest as f32
    } else {
        1.0
    }
}

fn resize_images<const COUNT: usize>(images: [(RgbImage, Size); COUNT]) -> [RgbImage; COUNT] {
    tracing::debug!("resizing {} images", COUNT);

    let span = tracing::Span::current();

    let images: Vec<_> = images
        .into_iter()
        .map(|(im, size)| {
            let span = span.clone();

            std::thread::spawn(move || {
                let _span = span.entered();
                resize_image(im, size)
            })
        })
        .collect::<Vec<_>>() // eagerly evaluate map to spawn threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();

    images.try_into().unwrap()
}

#[instrument(skip(image, size))]
fn resize_image(image: RgbImage, size: Size) -> RgbImage {
    tracing::trace!("starting image resize");

    let start = Instant::now();

    if image.width() != size.width && image.height() != size.height {
        let im = image::imageops::resize(
            &image,
            size.width,
            size.height,
            FilterType::Triangle, // The original uses Lanczos3 but in practice the difference is not visible.
        );

        tracing::debug!(time = start.elapsed().as_millis(), "resized image");

        im
    } else {
        tracing::debug!("image was already acceptable size");

        image
    }
}

#[derive(Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    fn scale(&self, scale_factor: f32) -> Size {
        Size {
            width: self.width / scale_factor,
            height: self.height / scale_factor,
        }
    }
    fn add(&self, other: Size) -> Size {
        Size {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

pub struct ImageOffset {
    pub offset: Size,
    pub dimensions: Size,
}

impl ImageOffset {
    fn scale(&self, scale_factor: f32) -> ImageOffset {
        ImageOffset {
            offset: self.offset.scale(scale_factor),
            dimensions: self.dimensions.scale(scale_factor),
        }
    }
}

trait MosaicDims {
    fn total_size(&self) -> Size;
    fn scale(&self, scale_factor: f32) -> MosaicDims;
}

pub struct Mosaic2ImageDims {
    pub image1: ImageOffset,
    pub image2: ImageOffset,
}

impl MosaicDims for Mosaic2Dims {
    fn total_size(&self) -> Size {
        self.image2.offset.add(self.image2.dimensions)
    }
    fn scale(&self, scale_factor: f32) -> Mosaic2Dims {
        Mosaic2Dims {
            image1: self.image1.scale(scale_factor),
            image2: self.image2.scale(scale_factor),
        }
    }
}

pub struct Mosaic3ImageDims {
    pub image1: ImageOffset,
    pub image2: ImageOffset,
    pub image3: ImageOffset,
}

impl MosaicDims for Mosaic3Dims {
    fn total_size(&self) -> Size {
        self.image3.offset.add(self.image3.dimensions)
    }
    fn scale(&self, scale_factor: f32) -> Mosaic3Dims {
        Mosaic3Dims {
            image1: self.image1.scale(scale_factor),
            image2: self.image2.scale(scale_factor),
            image3: self.image3.scale(scale_factor),
        }
    }
}

pub struct Mosaic4ImageDims {
    pub image1: ImageOffset,
    pub image2: ImageOffset,
    pub image3: ImageOffset,
    pub image4: ImageOffset,
}

impl MosaicDims for Mosaic3Dims {
    fn total_size(&self) -> Size {
        self.image4.offset.add(self.image4.dimensions)
    }
    fn scale(&self, scale_factor: f32) -> Mosaic3Dims {
        Mosaic3Dims {
            image1: self.image1.scale(scale_factor),
            image2: self.image2.scale(scale_factor),
            image3: self.image3.scale(scale_factor),
            image4: self.image4.scale(scale_factor),
        }
    }
}

pub struct HorizontalSize {
    pub width: u32,
    pub height: u32,
    pub first_width: u32,
    pub second_width: u32,
}

pub struct VerticalSize {
    pub width: u32,
    pub height: u32,
    pub first_height: u32,
    pub second_height: u32,
}

fn unsquareness(mosaic: MosaicDims) -> f32 {
    let total_size = mosaic.total_size()
    let ratio = if total_size.width < total_size.height {
        total_size.height as f32 / total_size.width as f32
    } else {
        total_size.width as f32 / total_size.height as f32
    }
    ratio
}

fn most_square_mosaic(mosaics: Vec<MosaicDims>) -> MosaicDims {
    mosaics.iter().min_by_key(|mosaic| {
        unsquareness(mosaic)
    });
}


fn best_2_mosaic(first: Size, second: Size) -> Mosaic2Dims {
    let top_bottom = top_bottom_2_mosaic(first, second);
    let left_right = left_right_2_mosaic(first, second);
    most_square_mosaic([top_bottom, left_right]);
}

fn build_2_mosaic(first: RgbImage, second: RgbImage) -> RgbImage {
    let first_size = Size {
        width: first.width(),
        height: first.height(),
    }
    let second_size = Size {
        width: second.width(),
        height: second.height(),
    }
    let best_mosaic = best_2_mosaic(first_size, second_size);
    let total_size = best_mosaic.total_size();
    let scale_factor = overall_scale_factor(total_size);
    best_mosaic = best_mosaic.scale(scale_factor);

    let [first, second] = resize_images([
        (
            first,
            best_mosaic.image1.dimensions,
        ),
        (
            second,
            best_mosaic.image2.dimensions,
        )
    ]);

    let mut background = create_background(
        best_mosaic.total_size()
    );
    let image::imageops::overlay(&mut background, &first, best_mosaic.image1.offset.width, best_mosaic.image1.offset.height);
    let image::imageops::overlay(&mut background, &second, best_mosaic.image2.offset.width, best_mosaic.image2.offset.height);
    background
}

fn left_right_2_mosaic(first: Size, second: Size) -> Mosaic2Dims {
    Mosaic2Dims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: first,
        },
        image2: ImageOffset {
            offset: Size {
                width: first.width + SPACING_SIZE,
                height: 0,
            },
            dimensions: scale_height_dimension(second, first.height),
        },
    }
}

fn top_bottom_2_mosaic(first: Size, second: Size) -> Mosaic2Dims {
    Mosaic2Dims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: first,
        },
        image2: ImageOffset {
            offset: Size {
                width: 0,
                height: first.height + SPACING_SIZE,
            },
            dimensions: scale_width_dimension(second, first.width),
        },
    }
}

fn best_3_mosaic(first: Size, second: Size, third: Size) -> Mosaic3Dims {
    let three_columns = three_columns_3_mosaic(first, second, third);
    let top_top_bottom = top_top_bottom_3_mosaic(first, second, third);
    let left_right_right = left_right_right_3_mosaic(first, second, third);
    let left_left_right = left_left_right_3_mosaic(first, second, third);
    let top_bottom_bottom = top_bottom_bottom_3_mosaic(first, second, third);
    let three_rows = three_rows_3_mosaic(first, second, third);
    most_square_mosaic([one_row, top_top_bottom, left_left_right, left_right_right, top_bottom_bottom, one_column]);
}

fn build_3_mosaic(first: RgbImage, second: RgbImage, third: RgbImage) -> RgbImage {
    let first_size = Size {
        width: first.width(),
        height: first.height(),
    }
    let second_size = Size {
        width: second.width(),
        height: second.height(),
    }
    let third_size = Size {
        width: third.width(),
        height: third.height(),
    }
    let best_mosaic = best_3_mosaic(first_size, second_size, third_size);
    let total_size = best_mosaic.total_size();
    let scale_factor = overall_scale_factor(total_size);
    best_mosaic = best_mosaic.scale(scale_factor);

    let [first, second, third] = resize_images([
        (
            first,
            best_mosaic.image1.dimensions,
        ),
        (
            second,
            best_mosaic.image2.dimensions,
        ),
        (
            third,
            best_mosaic.image3.dimensions,
        )
    ]);

    let mut background = create_background(
        best_mosaic.total_size()
    );
    let image::imageops::overlay(&mut background, &first, best_mosaic.image1.offset.width, best_mosaic.image1.offset.height);
    let image::imageops::overlay(&mut background, &second, best_mosaic.image2.offset.width, best_mosaic.image2.offset.height);
    let image::imageops::overlay(&mut background, &third, best_mosaic.image3.offset.width, best_mosaic.image3.offset.height);
    background
}

fn three_columns_3_mosaic(first: Size, second: Size, third: Size) -> Mosaic3Dims {
    let image2_offset = ImageOffset {
        offset: Size {
            width: first.width + SPACING_SIZE,
            height: 0
        },
        dimensions: scale_height_dimension(second, first.height),
    };

    Mosaic3Dims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: first,
        },
        image2: image2_offset,
        image3: ImageOffset {
            offset: Size {
                width: image2_offset.offset.width + image2_offset.dimensions.width + SPACING_SIZE,
                height: 0
            },
            dimensions: scale_height_dimension(third, first.height),
        },
    }
}

fn top_top_bottom_3_mosaic(first: Size, second: Size, third: Size) -> Mosaic3Dims {
    let image2_offset = ImageOffset {
        offset: Size {
            width: first.width + SPACING_SIZE,
            height: 0
        },
        dimensions: scale_height_dimension(second, first.height)
    };
    let total_width = image2_offset.offset.width + image2_offset.dimensions.width;

    Mosaic3Dims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: first,
        },
        image2: image2_offset,
        image3: ImageOffset {
            offset: Size {
                width: 0,
                height: first.height + SPACING_SIZE,
            },
            dimensions: scale_width_dimension(third, total_width),
        },
    }
}
