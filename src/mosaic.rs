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
            return build_2_mosaic(first, second)
        }
        3 => {
            let first = images.pop_front().unwrap();
            let second = images.pop_front().unwrap();
            let third = images.pop_front().unwrap();
            return build_3_mosaic(first, second, third)
        }
        4 => {
            let first = images.pop_front().unwrap();
            let second = images.pop_front().unwrap();
            let third = images.pop_front().unwrap();
            let fourth = images.pop_front().unwrap();
            return build_4_mosaic(first, second, third, fourth)
        }
        _ => panic!("impossible image length"),
    }
}

fn create_background(size: Size) -> RgbImage {
    RgbImage::from_pixel(size.width, size.height, image::Rgb([0, 0, 0]))
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
        width: (image_size.width as f32 / scale_factor).round() as u32,
        height: other_height,
    }
}

fn scale_width_dimension(image_size: Size, other_width: u32) -> Size {
    let scale_factor = image_size.width as f32 / other_width as f32;
    Size {
        width: other_width,
        height: (image_size.height as f32 / scale_factor).round() as u32,
    }
}

fn overall_scale_factor(size: Size) -> f32 {
    let biggest = if size.width > size.height {
        size.width
    } else {
        size.height
    };

    if biggest > MAX_SIZE {
        biggest as f32 / MAX_SIZE as f32
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
    fn add_height(&self, height: u32) -> ImageOffset {
        ImageOffset {
            offset: Size {
                width: self.offset.width,
                height: self.offset.height + height,
            },
            dimensions: self.dimensions,
        }
    }
    fn add_width(&self, width: u32) -> ImageOffset {
        ImageOffset {
            offset: Size {
                width: self.offset.width + width,
                height: self.offset.height,
            },
            dimensions: self.dimensions,
        }
    }
    fn total_width(&self) -> u32 {
        return self.offset.width + self.dimensions.width;
    }
    fn total_height(&self) -> u32 {
        return self.offset.height + self.dimensions.height;
    }
}

trait MosaicDims {
    fn total_size(&self) -> Size;
    fn scale(&self, scale_factor: f32) -> dyn MosaicDims;
    fn add_height(&self, height: u32) -> dyn MosaicDims;
    fn add_width(&self, width: u32) -> dyn MosaicDims;
}

pub struct Mosaic2ImageDims {
    pub image1: ImageOffset,
    pub image2: ImageOffset,
}

impl MosaicDims for Mosaic2ImageDims {
    fn total_size(&self) -> Size {
        self.image2.offset.add(self.image2.dimensions)
    }
    fn scale(&self, scale_factor: f32) -> Mosaic2ImageDims {
        Mosaic2ImageDims {
            image1: self.image1.scale(scale_factor),
            image2: self.image2.scale(scale_factor),
        }
    }
    fn add_height(&self, height: u32) -> Mosaic2ImageDims {
        Mosaic2ImageDims {
            image1: self.image1.add_height(height),
            image2: self.image2.add_height(height),
        }
    }
    fn add_width(&self, width: u32) -> Mosaic2ImageDims {
        Mosaic2ImageDims {
            image1: self.image1.add_width(width),
            image2: self.image2.add_width(width),
        }
    }
}

pub struct Mosaic3ImageDims {
    pub image1: ImageOffset,
    pub image2: ImageOffset,
    pub image3: ImageOffset,
}

impl MosaicDims for Mosaic3ImageDims {
    fn total_size(&self) -> Size {
        self.image3.offset.add(self.image3.dimensions)
    }
    fn scale(&self, scale_factor: f32) -> Mosaic3ImageDims {
        Mosaic3ImageDims {
            image1: self.image1.scale(scale_factor),
            image2: self.image2.scale(scale_factor),
            image3: self.image3.scale(scale_factor),
        }
    }
    fn add_height(&self, height: u32) -> Mosaic3ImageDims {
        Mosaic3ImageDims {
            image1: self.image1.add_height(height),
            image2: self.image2.add_height(height),
            image3: self.image3.add_height(height),
        }
    }
    fn add_width(&self, width: u32) -> Mosaic3ImageDims {
        Mosaic3ImageDims {
            image1: self.image1.add_width(width),
            image2: self.image2.add_width(width),
            image3: self.image3.add_width(width),
        }
    }
}

pub struct Mosaic4ImageDims {
    pub image1: ImageOffset,
    pub image2: ImageOffset,
    pub image3: ImageOffset,
    pub image4: ImageOffset,
}

impl MosaicDims for Mosaic4ImageDims {
    fn total_size(&self) -> Size {
        self.image4.offset.add(self.image4.dimensions)
    }
    fn scale(&self, scale_factor: f32) -> Mosaic4ImageDims {
        Mosaic4ImageDims {
            image1: self.image1.scale(scale_factor),
            image2: self.image2.scale(scale_factor),
            image3: self.image3.scale(scale_factor),
            image4: self.image4.scale(scale_factor),
        }
    }
    fn add_height(&self, height: u32) -> Mosaic4ImageDims {
        Mosaic4ImageDims {
            image1: self.image1.add_height(height),
            image2: self.image2.add_height(height),
            image3: self.image3.add_height(height),
            image4: self.image4.add_height(height),
        }
    }
    fn add_width(&self, width: u32) -> Mosaic4ImageDims {
        Mosaic4ImageDims {
            image1: self.image1.add_width(width),
            image2: self.image2.add_width(width),
            image3: self.image3.add_width(width),
            image4: self.image4.add_width(width),
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

fn unsquareness<T: MosaicDims>(mosaic: T) -> f32 {
    let total_size = mosaic.total_size();
    let ratio = if total_size.width < total_size.height {
        total_size.height as f32 / total_size.width as f32
    } else {
        total_size.width as f32 / total_size.height as f32
    };
    ratio
}

fn most_square_mosaic<T: MosaicDims>(mosaics: &[T]) -> T {
    mosaics.iter().min_by_key(|mosaic| {
        unsquareness(mosaic)
    })
}

fn best_2_mosaic(first: Size, second: Size) -> Mosaic2ImageDims {
    let top_bottom = top_bottom_2_mosaic(first, second);
    let left_right = left_right_2_mosaic(first, second);
    return most_square_mosaic(&[top_bottom, left_right]);
}

fn build_2_mosaic(first: RgbImage, second: RgbImage) -> RgbImage {
    let first_size = Size {
        width: first.width(),
        height: first.height(),
    };
    let second_size = Size {
        width: second.width(),
        height: second.height(),
    };
    let best_mosaic = best_2_mosaic(first_size, second_size);
    let total_size = best_mosaic.total_size();
    let scale_factor = overall_scale_factor(total_size);
    let scaled_mosaic = best_mosaic.scale(scale_factor);

    let [first, second] = resize_images([
        (
            first,
            scaled_mosaic.image1.dimensions,
        ),
        (
            second,
            scaled_mosaic.image2.dimensions,
        )
    ]);

    let mut background = create_background(scaled_mosaic.total_size());
    image::imageops::overlay(&mut background, &first, scaled_mosaic.image1.offset.width as i64, scaled_mosaic.image1.offset.height as i64);
    image::imageops::overlay(&mut background, &second, scaled_mosaic.image2.offset.width as i64, scaled_mosaic.image2.offset.height as i64);
    background
}

fn left_right_2_mosaic(first: Size, second: Size) -> Mosaic2ImageDims {
    Mosaic2ImageDims {
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

fn top_bottom_2_mosaic(first: Size, second: Size) -> Mosaic2ImageDims {
    Mosaic2ImageDims {
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

fn best_3_mosaic(first: Size, second: Size, third: Size) -> Mosaic3ImageDims {
    let three_columns = three_columns_3_mosaic(first, second, third);
    let top_top_bottom = top_top_bottom_3_mosaic(first, second, third);
    let left_right_right = left_right_right_3_mosaic(first, second, third);
    let left_left_right = left_left_right_3_mosaic(first, second, third);
    let top_bottom_bottom = top_bottom_bottom_3_mosaic(first, second, third);
    let three_rows = three_rows_3_mosaic(first, second, third);
    return most_square_mosaic(&[three_columns, top_top_bottom, left_left_right, left_right_right, top_bottom_bottom, three_rows]);
}

fn build_3_mosaic(first: RgbImage, second: RgbImage, third: RgbImage) -> RgbImage {
    let first_size = Size {
        width: first.width(),
        height: first.height(),
    };
    let second_size = Size {
        width: second.width(),
        height: second.height(),
    };
    let third_size = Size {
        width: third.width(),
        height: third.height(),
    };
    let best_mosaic = best_3_mosaic(first_size, second_size, third_size);
    let total_size = best_mosaic.total_size();
    let scale_factor = overall_scale_factor(total_size);
    let scaled_mosaic = best_mosaic.scale(scale_factor);

    let [first, second, third] = resize_images([
        (
            first,
            scaled_mosaic.image1.dimensions,
        ),
        (
            second,
            scaled_mosaic.image2.dimensions,
        ),
        (
            third,
            scaled_mosaic.image3.dimensions,
        )
    ]);

    let mut background = create_background(scaled_mosaic.total_size());
    image::imageops::overlay(&mut background, &first, scaled_mosaic.image1.offset.width as i64, scaled_mosaic.image1.offset.height as i64);
    image::imageops::overlay(&mut background, &second, scaled_mosaic.image2.offset.width as i64, scaled_mosaic.image2.offset.height as i64);
    image::imageops::overlay(&mut background, &third, scaled_mosaic.image3.offset.width as i64, scaled_mosaic.image3.offset.height as i64);
    background
}

fn three_columns_3_mosaic(first: Size, second: Size, third: Size) -> Mosaic3ImageDims {
    let image2_offset = ImageOffset {
        offset: Size {
            width: first.width + SPACING_SIZE,
            height: 0
        },
        dimensions: scale_height_dimension(second, first.height),
    };

    Mosaic3ImageDims {
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
                width: image2_offset.total_width() + SPACING_SIZE,
                height: 0
            },
            dimensions: scale_height_dimension(third, first.height),
        },
    }
}

fn top_top_bottom_3_mosaic(first: Size, second: Size, third: Size) -> Mosaic3ImageDims {
    let image2_offset = ImageOffset {
        offset: Size {
            width: first.width + SPACING_SIZE,
            height: 0
        },
        dimensions: scale_height_dimension(second, first.height)
    };

    Mosaic3ImageDims {
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
            dimensions: scale_width_dimension(third, image2_offset.total_width()),
        },
    }
}

fn left_left_right_3_mosaic(first: Size, second: Size, third: Size) -> Mosaic3ImageDims {
    let image2_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: first.height + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(second, first.width)
    };

    Mosaic3ImageDims {
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
                width: first.width + SPACING_SIZE,
                height: 0,
            },
            dimensions: scale_height_dimension(third, image2_offset.total_height()),
        },
    }
}

fn left_right_right_3_mosaic(first: Size, second: Size, third: Size) -> Mosaic3ImageDims {
    let image3_dims = scale_width_dimension(third, second.width);
    let image1_dims = scale_height_dimension(first, second.height + image3_dims.height + SPACING_SIZE);
    let image2_offset = ImageOffset {
        offset: Size {
            width: image1_dims.width + SPACING_SIZE,
            height: 0,
        },
        dimensions: second,
    };
    let image3_offset = ImageOffset {
        offset: Size {
            width: image1_dims.width + SPACING_SIZE,
            height: image2_offset.total_height() + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(third, second.width),
    };

    Mosaic3ImageDims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: image1_dims,
        },
        image2: image2_offset,
        image3: image3_offset,
    }
}

fn top_bottom_bottom_3_mosaic(first: Size, second: Size, third: Size) -> Mosaic3ImageDims {
    let image3_dims = scale_height_dimension(third, second.height);
    let image1_dims = scale_width_dimension(first, second.width + image3_dims.width + SPACING_SIZE);

    Mosaic3ImageDims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: image1_dims,
        },
        image2: ImageOffset {
            offset: Size {
                width: 0,
                height: image1_dims.height + SPACING_SIZE,
            },
            dimensions: second,
        },
        image3: ImageOffset {
            offset: Size {
                width: second.width + SPACING_SIZE,
                height: image1_dims.height + SPACING_SIZE,
            },
            dimensions: image3_dims,
        },
    }
}

fn three_rows_3_mosaic(first: Size, second: Size, third: Size) -> Mosaic3ImageDims {
    let image2_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: first.height + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(second, first.width),
    };

    Mosaic3ImageDims {
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
                height: image2_offset.total_height() + SPACING_SIZE,
            },
            dimensions: scale_width_dimension(third, first.width),
        },
    }
}

fn best_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let four_columns = four_columns_4_mosaic(first, second, third, fourth);
    let four_rows = four_rows_4_mosaic(first, second, third, fourth);
    let two_rows_of_two = two_rows_of_two_4_mosaic(first, second, third, fourth);
    let two_rows_one_three = two_rows_one_three_4_mosaic(first, second, third, fourth);
    let two_rows_three_one = two_rows_three_one_4_mosaic(first, second, third, fourth);
    let two_columns_of_two = two_columns_of_two_4_mosaic(first, second, third, fourth);
    let two_columns_one_three = two_columns_one_three(first, second, third, fourth);
    let two_columns_three_one = two_columns_three_one(first, second, third, fourth);
    let three_rows_211 = three_rows_211_4_mosaic(first, second, third, fourth);
    let three_rows_121 = three_rows_121_4_mosaic(first, second, third, fourth);
    let three_rows_112 = three_rows_112_4_mosaic(first, second, third, fourth);
    let three_columns_211 = three_columns_211_4_mosaic(first, second, third, fourth);
    let three_columns_121 = three_columns_121_4_mosaic(first, second, third, fourth);
    let three_columns_112 = three_columns_112_4_mosaic(first, second, third, fourth);
    return most_square_mosaic(&[four_columns, four_rows, two_rows_of_two, two_rows_one_three, two_rows_three_one, two_columns_of_two, two_columns_one_three, two_columns_three_one, three_rows_211, three_rows_121, three_rows_112, three_columns_211, three_columns_121, three_columns_112]);
}

fn build_4_mosaic(first: RgbImage, second: RgbImage, third: RgbImage, fourth: RgbImage) -> RgbImage {
    let first_size = Size {
        width: first.width(),
        height: first.height(),
    };
    let second_size = Size {
        width: second.width(),
        height: second.height(),
    };
    let third_size = Size {
        width: third.width(),
        height: third.height(),
    };
    let fourth_size = Size {
        width: fourth.width(),
        height: fourth.height(),
    };
    let best_mosaic = best_4_mosaic(first_size, second_size, third_size, fourth_size);
    let total_size = best_mosaic.total_size();
    let scale_factor = overall_scale_factor(total_size);
    let scaled_mosaic = best_mosaic.scale(scale_factor);

    let [first, second, third, fourth] = resize_images([
        (
            first,
            scaled_mosaic.image1.dimensions,
        ),
        (
            second,
            scaled_mosaic.image2.dimensions,
        ),
        (
            third,
            scaled_mosaic.image3.dimensions,
        ),
        (
            fourth,
            scaled_mosaic.image4.dimensions,
        ),
    ]);

    let mut background = create_background(scaled_mosaic.total_size());
    image::imageops::overlay(&mut background, &first, scaled_mosaic.image1.offset.width as i64, scaled_mosaic.image1.offset.height as i64);
    image::imageops::overlay(&mut background, &second, scaled_mosaic.image2.offset.width as i64, scaled_mosaic.image2.offset.height as i64);
    image::imageops::overlay(&mut background, &third, scaled_mosaic.image3.offset.width as i64, scaled_mosaic.image3.offset.height as i64);
    image::imageops::overlay(&mut background, &fourth, scaled_mosaic.image4.offset.width as i64, scaled_mosaic.image4.offset.height as i64);
    background
}

fn four_columns_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let image2_offset = ImageOffset {
        offset: Size {
            width: first.width + SPACING_SIZE,
            height: 0,
        },
        dimensions: scale_height_dimension(second, first.height),
    };
    let image3_offset = ImageOffset {
        offset: Size {
            width: image2_offset.total_width() + SPACING_SIZE,
            height: 0,
        },
        dimensions: scale_height_dimension(third, first.height),
    };

    Mosaic4ImageDims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: first,
        },
        image2: image2_offset,
        image3: image3_offset,
        image4: ImageOffset {
            offset: Size {
                width: image3_offset.total_width() + SPACING_SIZE,
                height: 0,
            },
            dimensions: scale_height_dimension(fourth, first.height),
        },
    }
}

fn four_rows_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let image2_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: first.height + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(second, first.width),
    };
    let image3_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: image2_offset.total_height() + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(third, first.width)
    };

    Mosaic4ImageDims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: first,
        },
        image2: image2_offset,
        image3: image3_offset,
        image4: ImageOffset {
            offset: Size {
                width: 0,
                height: image3_offset.total_height() + SPACING_SIZE,
            },
            dimensions: scale_width_dimension(fourth, first.width),
        },
    }
}

fn two_rows_of_two_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let first_row = left_right_2_mosaic(first, second);
    let second_row = left_right_2_mosaic(third, fourth);
    let scale_factor = second_row.total_size().width as f32 / first_row.total_size().width as f32;
    let second_row_moved = second_row.scale(scale_factor).add_height(first_row.total_size().height + SPACING_SIZE);

    Mosaic4ImageDims {
        image1: first_row.image1,
        image2: first_row.image2,
        image3: second_row_moved.image1,
        image4: second_row_moved.image2,
    }
}

fn two_rows_one_three_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let second_row = three_columns_3_mosaic(second, third, fourth);
    let image1_dims = scale_width_dimension(first, second_row.total_size().width);
    let second_row_moved = second_row.add_height(image1_dims.height + SPACING_SIZE);


    Mosaic4ImageDims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: image1_dims,
        },
        image2: second_row_moved.image1,
        image3: second_row_moved.image2,
        image4: second_row_moved.image3,
    }
}

fn two_rows_three_one_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let first_row = three_columns_3_mosaic(first, second, third);
    let image4_dims = scale_width_dimension(fourth, first_row.total_size().width);

    Mosaic4ImageDims {
        image1: first_row.image1,
        image2: first_row.image2,
        image3: first_row.image3,
        image4: ImageOffset {
            offset: Size {
                width: 0,
                height: first_row.total_size().height + SPACING_SIZE,
            },
            dimensions: image4_dims,
        },
    }
}

fn two_columns_of_two_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let first_col = top_bottom_2_mosaic(first, second);
    let second_col = top_bottom_2_mosaic(third, fourth);
    let scale_factor = second_col.total_size().height as f32 / first_col.total_size().height as f32;
    let second_col_moved = second_col.scale(scale_factor).add_width(first_col.total_size().width + SPACING_SIZE);

    Mosaic4ImageDims {
        image1: first_col.image1,
        image2: first_col.image2,
        image3: second_col_moved.image1,
        image4: second_col_moved.image2,
    }
}

fn two_columns_one_three_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let second_col = three_rows_3_mosaic(second, third, fourth);
    let image1_dims = scale_height_dimension(first, second_col.total_size().height);
    let second_col_moved = second_col.add_width(image1_dims.width + SPACING_SIZE);

    Mosaic4ImageDims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: image1_dims,
        },
        image2: second_col_moved.image1,
        image3: second_col_moved.image2,
        image4: second_col_moved.image3,
    }
}

fn two_columns_three_one_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let first_col = three_rows_3_mosaic(first, second, third);
    let image4_dims = scale_height_dimension(fourth, first_col.total_size().height);

    Mosaic4ImageDims {
        image1: first_row.image1,
        image2: first_row.image2,
        image3: first_row.image3,
        image4: ImageOffset {
            offset: Size {
                width: first_col.total_size().width + SPACING_SIZE,
                height: 0,
            },
            dimensions: image4_dims,
        },
    }
}

fn three_rows_211_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let first_row = left_right_2_mosaic(first, second);
    let image3_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: first_row.total_size().height + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(third, first_row.total_size().width),
    };

    Mosaic4ImageDims {
        image1: first_row.image1,
        image2: first_row.image2,
        image3: image3_offset,
        image4: ImageOffset {
            offset: Size {
                width: 0,
                height: image3_offset.total_height(),
            },
            dimensions: scale_width_dimension(fourth, first_row.total_size().width),
        }
    }
}

fn three_rows_121_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let second_row = left_right_2_mosaic(second, third);
    image1_dims = scale_width_dimension(first, second_row.total_size().width);
    second_row.add_height(image1_dims.total_height() + SPACING_SIZE);

    Mosaic4ImageDims {
        image1: ImageOffset {
            offset: Size {
                width: 0,
                height: 0,
            },
            dimensions: image1_dims,
        },
        image2: second_row.image1,
        image3: second_row.image2,
        image4: ImageOffset {
            offset: Size {
                width: 0,
                height: second_row.total_size().height + SPACING_SIZE,
            },
            dimensions: scale_width_dimension(fourth, second.total_size().width),
        },
    }
}

fn three_rows_112_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let third_row = left_right_2_mosaic(third, fourth);
    image1_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: 0,
        },
        dimensions: scale_width_dimension(first, third_row.total_size().width),
    };
    image2_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: image1_offset.total_height() + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(second, third_row.total_size().width),
    };
    let third_row_moved = third_row.add_height(image2_offset.total_height() + SPACING_SIZE);

    Mosaic4ImageDims {
        image1: image1_offset,
        image2: image2_offset,
        image3: third_row_moved.image1,
        image4: third_row_moved.image2,
    }
}

fn three_columns_211_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let first_col = top_bottom_2_mosaic(first, second);
    let image3_offset = ImageOffset {
        offset: Size {
            width: first_col.total_size().width + SPACING_SIZE,
            height: 0,
        },
        dimensions: scale_height_dimension(third, first_col.total_size().height),
    };

    Mosaic4ImageDims {
        image1: first_col.image1,
        image2: first_col.image2,
        image3: image3_offset,
        image4: ImageOffset {
            offset: Size {
                width: image3_offset.total_width() + SPACING_SIZE,
                height: 0,
            },
            dimensions: scale_height_dimension(fourth, first_col.total_size().height),
        },
    }
}

fn three_columns_121_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let second_col = top_bottom_2_mosaic(second, third);
    let image1_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: 0,
        },
        dimensions: scale_height_dimension(first, second_col.total_size().height),
    };
    let second_col_moved = second_col.add_width(image1_offset.total_width() + SPACING_SIZE);

    Mosaic4ImageDims {
        image1: image1_offset,
        image2: second_col_moved.image1,
        image3: second_col_moved.image2,
        image4: ImageOffset {
            offset: Size {
                width: second_col_moved.total_size().width + SPACING_SIZE,
                height: 0,
            },
            dimensions: scale_height_dimension(fourth, second_col_moved.total_size().height),
        },
    }
}

fn three_columns_112_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> Mosaic4ImageDims {
    let third_col = top_bottom_2_mosaic(third, fourth);
    let image1_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: 0,
        },
        dimensions: scale_height_dimension(first, third_col.total_size().height),
    };
    let image2_offset = ImageOffset {
        offset: Size {
            width: image1_offset.total_width() + SPACING_SIZE,
            height: 0,
        },
        dimensions: scale_height_dimension(second, third_col.total_size().height),
    };
    let third_col_moved = third_col.add_width(image2_offset.total_width() + SPACING_SIZE);

    Mosaic4ImageDims {
        image1: image1_offset,
        image2: image2_offset,
        image3: third_col_moved.image1,
        image4: third_col_moved.image2,
    }
}
