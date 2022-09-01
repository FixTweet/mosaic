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

use std::cmp::max;
use std::time::Instant;
use std::cmp::Ordering::Equal;
use std::iter::zip;

use image::{imageops::FilterType, RgbImage};
use tracing::instrument;

const SPACING_SIZE: u32 = 10;
const MAX_SIZE: u32 = 4000;

pub fn mosaic(mut images: Vec<RgbImage>) -> RgbImage {
    match images.len() {
        2 => {
            let second = images.pop().unwrap();
            let first = images.pop().unwrap();
            build_2_mosaic(first, second)
        }
        3 => {
            let third = images.pop().unwrap();
            let second = images.pop().unwrap();
            let first = images.pop().unwrap();
            build_3_mosaic(first, second, third)
        }
        4 => {
            let fourth = images.pop().unwrap();
            let third = images.pop().unwrap();
            let second = images.pop().unwrap();
            let first = images.pop().unwrap();
            build_4_mosaic(first, second, third, fourth)
        }
        _ => panic!("impossible image length"),
    }
}

fn create_background(size: Size) -> RgbImage {
    RgbImage::from_pixel(size.width, size.height, image::Rgb([0, 0, 0]))
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

fn resize_images(images: Vec<(RgbImage, Size)>) -> Vec<RgbImage> {
    tracing::debug!("resizing {} images", images.len());

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

    images
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

#[derive(Clone, Copy, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    fn scale(&self, scale_factor: f32) -> Size {
        Size {
            width: (self.width as f32 / scale_factor).round() as u32,
            height: (self.height as f32 / scale_factor).round() as u32,
        }
    }
    fn add(&self, other: Size) -> Size {
        Size {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct ImageOffset {
    pub offset: Size,
    pub dimensions: Size,
    pub original_dimensions: Size,
}

impl ImageOffset {
    fn scale(&self, scale_factor: f32) -> ImageOffset {
        ImageOffset {
            offset: self.offset.scale(scale_factor),
            dimensions: self.dimensions.scale(scale_factor),
            original_dimensions: self.original_dimensions,
        }
    }
    fn add_height(&self, height: u32) -> ImageOffset {
        ImageOffset {
            offset: Size {
                width: self.offset.width,
                height: self.offset.height + height,
            },
            dimensions: Size {
                width: self.dimensions.width,
                height: self.dimensions.height,
            },
            original_dimensions: self.original_dimensions,
        }
    }
    fn add_width(&self, width: u32) -> ImageOffset {
        ImageOffset {
            offset: Size {
                width: self.offset.width + width,
                height: self.offset.height,
            },
            dimensions: Size {
                width: self.dimensions.width,
                height: self.dimensions.height,
            },
            original_dimensions: self.original_dimensions,
        }
    }
    fn total_width(&self) -> u32 {
        self.offset.width + self.dimensions.width
    }
    fn total_height(&self) -> u32 {
        self.offset.height + self.dimensions.height
    }
}

trait MosaicDims {
    fn total_size(&self) -> Size;
    fn scale(&self, scale_factor: f32) -> Self;
    fn image_scale_factors(&self) -> Vec<f32>;
    fn min_scale_factor(&self) -> f32;
    fn max_scale_factor(&self) -> f32;
    fn scale_factor_ratio(&self) -> f32;
    fn scale_to_fit(&self) -> Self;
    fn add_height(&self, height: u32) -> Self;
    fn add_width(&self, width: u32) -> Self;

    fn unsquaredness(&self) -> f32 {
        let total_size = self.total_size();
        if total_size.width < total_size.height {
            total_size.height as f32 / total_size.width as f32
        } else {
            total_size.width as f32 / total_size.height as f32
        }
    }
}

#[derive(Clone, Copy)]
pub struct MosaicImageDims<const LEN: usize> {
    images: [ImageOffset; LEN]
}

impl<const LEN: usize> MosaicDims for MosaicImageDims<LEN> {
    fn total_size(&self) -> Size {
        let last = self.images.last().unwrap();
        last.offset.add(last.dimensions)
    }
    fn scale(&self, scale_factor: f32) -> Self {
        let mut new_images = [ImageOffset::default(); LEN];
        for (x, image) in self.images.iter().enumerate() {
            new_images[x] = image.scale(scale_factor);
        }
        MosaicImageDims {
            images: new_images
        }
    }
    fn image_scale_factors(&self) -> Vec<f32> {
        self.images.iter().map(|image| {
            image.dimensions.width as f32 / image.original_dimensions.width as f32
        }).collect()
    }
    fn min_scale_factor(&self) -> f32 {
        *(self.image_scale_factors().iter().min_by(|a, b| {
            a.partial_cmp(&b).unwrap_or(Equal)
        }).unwrap())
    }
    fn max_scale_factor(&self) -> f32 {
        *self.image_scale_factors().iter().max_by(|a, b| {
            a.partial_cmp(&b).unwrap_or(Equal)
        }).unwrap()
    }
    fn scale_factor_ratio(&self) -> f32 {
        self.max_scale_factor() / self.min_scale_factor()
    }
    fn scale_to_fit(&self) -> Self {
        // Scale mosaic so that the smallest image is 1:1 scale
        let mut scaled_mosaic = self.scale(self.min_scale_factor());
        // Scale down to fit into maximum dimensions
        let total_size = scaled_mosaic.total_size();
        let biggest = max(total_size.width, total_size.height);
        if biggest > MAX_SIZE {
            let scale_factor = biggest as f32 / MAX_SIZE as f32;
            scaled_mosaic = scaled_mosaic.scale(scale_factor);
            let new_size = scaled_mosaic.total_size();
        }
        scaled_mosaic
    }
    fn add_height(&self, height: u32) -> Self {
        let mut new_images = [ImageOffset::default(); LEN];
        for (x, image) in self.images.iter().enumerate() {
            new_images[x] = image.add_height(height);
        }
        MosaicImageDims {
            images: new_images
        }
    }
    fn add_width(&self, width: u32) -> Self {
        let mut new_images = [ImageOffset::default(); LEN];
        for (x, image) in self.images.iter().enumerate() {
            new_images[x] = image.add_width(width);
        }
        MosaicImageDims {
            images: new_images
        }
    }
}

fn most_square_mosaic<'b,T: MosaicDims>(mosaics: &[&'b T]) -> &'b T {
    mosaics.iter().min_by(|mosaic_a, mosaic_b| {
        let ratio_a = mosaic_a.unsquaredness();
        let ratio_b = mosaic_b.unsquaredness();
        ratio_a.partial_cmp(&ratio_b).unwrap_or(Equal)
    }).unwrap()
}

fn best_2_mosaic(first: Size, second: Size) -> MosaicImageDims<2> {
    let top_bottom = top_bottom_2_mosaic(first, second);
    let left_right = left_right_2_mosaic(first, second);
    return *(most_square_mosaic(&[&top_bottom, &left_right]));
}

fn build_mosaic<const LEN: usize>(mosaic: MosaicImageDims<LEN>, images: [RgbImage; LEN]) -> RgbImage {
    let resize_args = zip(images, mosaic.images).map(|(image, offset)| {
        (
            image,
            offset.dimensions,
        )
    }).collect();

    let resized = resize_images(resize_args);

    let mut background = create_background(mosaic.total_size());
    for (image, offset) in zip(resized, mosaic.images) {
        image::imageops::overlay(&mut background, &image, offset.offset.width as i64, offset.offset.height as i64);
    }
    background
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
    build_mosaic(best_mosaic, [first, second])
}

fn left_right_2_mosaic(first: Size, second: Size) -> MosaicImageDims<2> {
    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
                original_dimensions: first,
            },
            ImageOffset {
                offset: Size {
                    width: first.width + SPACING_SIZE,
                    height: 0,
                },
                dimensions: scale_height_dimension(second, first.height),
                original_dimensions: second,
            },
        ]
    }
}

fn top_bottom_2_mosaic(first: Size, second: Size) -> MosaicImageDims<2> {
    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
                original_dimensions: first,
            },
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: first.height + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(second, first.width),
                original_dimensions: second,
            },
        ]
    }
}

fn best_3_mosaic(first: Size, second: Size, third: Size) -> MosaicImageDims<3> {
    let three_columns = three_columns_3_mosaic(first, second, third);
    let top_top_bottom = top_top_bottom_3_mosaic(first, second, third);
    let left_right_right = left_right_right_3_mosaic(first, second, third);
    let left_left_right = left_left_right_3_mosaic(first, second, third);
    let top_bottom_bottom = top_bottom_bottom_3_mosaic(first, second, third);
    let three_rows = three_rows_3_mosaic(first, second, third);
    return *(most_square_mosaic(&[&three_columns, &top_top_bottom, &left_left_right, &left_right_right, &top_bottom_bottom, &three_rows]));
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
    build_mosaic(best_mosaic, [first, second, third])
}

fn three_columns_3_mosaic(first: Size, second: Size, third: Size) -> MosaicImageDims<3> {
    let image2_offset = ImageOffset {
        offset: Size {
            width: first.width + SPACING_SIZE,
            height: 0
        },
        dimensions: scale_height_dimension(second, first.height),
        original_dimensions: second,
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
                original_dimensions: first,
            },
            image2_offset,
            ImageOffset {
                offset: Size {
                    width: image2_offset.total_width() + SPACING_SIZE,
                    height: 0
                },
                dimensions: scale_height_dimension(third, first.height),
                original_dimensions: third,
            },
        ],
    }
}

fn top_top_bottom_3_mosaic(first: Size, second: Size, third: Size) -> MosaicImageDims<3> {
    let image2_offset = ImageOffset {
        offset: Size {
            width: first.width + SPACING_SIZE,
            height: 0
        },
        dimensions: scale_height_dimension(second, first.height),
        original_dimensions: second,
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
                original_dimensions: first,
            },
            image2_offset,
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: first.height + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(third, image2_offset.total_width()),
                original_dimensions: third,
            },
        ],
    }
}

fn left_left_right_3_mosaic(first: Size, second: Size, third: Size) -> MosaicImageDims<3> {
    let image2_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: first.height + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(second, first.width),
        original_dimensions: second,
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
                original_dimensions: first,
            },
            image2_offset,
            ImageOffset {
                offset: Size {
                    width: first.width + SPACING_SIZE,
                    height: 0,
                },
                dimensions: scale_height_dimension(third, image2_offset.total_height()),
                original_dimensions: third,
            },
        ],
    }
}

fn left_right_right_3_mosaic(first: Size, second: Size, third: Size) -> MosaicImageDims<3> {
    let image3_dims = scale_width_dimension(third, second.width);
    let image1_dims = scale_height_dimension(first, second.height + image3_dims.height + SPACING_SIZE);
    let image2_offset = ImageOffset {
        offset: Size {
            width: image1_dims.width + SPACING_SIZE,
            height: 0,
        },
        dimensions: second,
        original_dimensions: second,
    };
    let image3_offset = ImageOffset {
        offset: Size {
            width: image1_dims.width + SPACING_SIZE,
            height: image2_offset.total_height() + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(third, second.width),
        original_dimensions: third,
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: image1_dims,
                original_dimensions: first,
            },
            image2_offset,
            image3_offset,
        ],
    }
}

fn top_bottom_bottom_3_mosaic(first: Size, second: Size, third: Size) -> MosaicImageDims<3> {
    let image3_dims = scale_height_dimension(third, second.height);
    let image1_dims = scale_width_dimension(first, second.width + image3_dims.width + SPACING_SIZE);

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: image1_dims,
                original_dimensions: first,
            },
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: image1_dims.height + SPACING_SIZE,
                },
                dimensions: second,
                original_dimensions: second,
            },
            ImageOffset {
                offset: Size {
                    width: second.width + SPACING_SIZE,
                    height: image1_dims.height + SPACING_SIZE,
                },
                dimensions: image3_dims,
                original_dimensions: third,
            },
        ],
    }
}

fn three_rows_3_mosaic(first: Size, second: Size, third: Size) -> MosaicImageDims<3> {
    let image2_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: first.height + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(second, first.width),
        original_dimensions: second,
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
                original_dimensions: first,
            },
            image2_offset,
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: image2_offset.total_height() + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(third, first.width),
                original_dimensions: third,
            },
        ],
    }
}

fn best_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let four_columns = four_columns_4_mosaic(first, second, third, fourth);
    let four_rows = four_rows_4_mosaic(first, second, third, fourth);
    let two_rows_of_two = two_rows_of_two_4_mosaic(first, second, third, fourth);
    let two_rows_one_three = two_rows_one_three_4_mosaic(first, second, third, fourth);
    let two_rows_three_one = two_rows_three_one_4_mosaic(first, second, third, fourth);
    let two_columns_of_two = two_columns_of_two_4_mosaic(first, second, third, fourth);
    let two_columns_one_three = two_columns_one_three_4_mosaic(first, second, third, fourth);
    let two_columns_three_one = two_columns_three_one_4_mosaic(first, second, third, fourth);
    let three_rows_211 = three_rows_211_4_mosaic(first, second, third, fourth);
    let three_rows_121 = three_rows_121_4_mosaic(first, second, third, fourth);
    let three_rows_112 = three_rows_112_4_mosaic(first, second, third, fourth);
    let three_columns_211 = three_columns_211_4_mosaic(first, second, third, fourth);
    let three_columns_121 = three_columns_121_4_mosaic(first, second, third, fourth);
    let three_columns_112 = three_columns_112_4_mosaic(first, second, third, fourth);
    return *(most_square_mosaic(&[&four_columns, &four_rows, &two_rows_of_two, &two_rows_one_three, &two_rows_three_one, &two_columns_of_two, &two_columns_one_three, &two_columns_three_one, &three_rows_211, &three_rows_121, &three_rows_112, &three_columns_211, &three_columns_121, &three_columns_112]));
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
    build_mosaic(best_mosaic, [first, second, third, fourth])
}

fn four_columns_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let image2_offset = ImageOffset {
        offset: Size {
            width: first.width + SPACING_SIZE,
            height: 0,
        },
        dimensions: scale_height_dimension(second, first.height),
        original_dimensions: second,
    };
    let image3_offset = ImageOffset {
        offset: Size {
            width: image2_offset.total_width() + SPACING_SIZE,
            height: 0,
        },
        dimensions: scale_height_dimension(third, first.height),
        original_dimensions: third,
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
                original_dimensions: first,
            },
            image2_offset,
            image3_offset,
            ImageOffset {
                offset: Size {
                    width: image3_offset.total_width() + SPACING_SIZE,
                    height: 0,
                },
                dimensions: scale_height_dimension(fourth, first.height),
                original_dimensions: fourth,
            },
        ],
    }
}

fn four_rows_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let image2_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: first.height + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(second, first.width),
        original_dimensions: second,
    };
    let image3_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: image2_offset.total_height() + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(third, first.width),
        original_dimensions: third,
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
                original_dimensions: first,
            },
            image2_offset,
            image3_offset,
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: image3_offset.total_height() + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(fourth, first.width),
                original_dimensions: fourth,
            },
        ],
    }
}

fn two_rows_of_two_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let first_row = left_right_2_mosaic(first, second);
    let second_row = left_right_2_mosaic(third, fourth);
    let scale_factor = second_row.total_size().width as f32 / first_row.total_size().width as f32;
    let second_row_moved = second_row.scale(scale_factor).add_height(first_row.total_size().height + SPACING_SIZE);

    MosaicImageDims {
        images: [
            first_row.images[0],
            first_row.images[1],
            second_row_moved.images[0],
            second_row_moved.images[1],
        ],
    }
}

fn two_rows_one_three_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let second_row = three_columns_3_mosaic(second, third, fourth);
    let image1_dims = scale_width_dimension(first, second_row.total_size().width);
    let second_row_moved = second_row.add_height(image1_dims.height + SPACING_SIZE);

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: image1_dims,
                original_dimensions: first,
            },
            second_row_moved.images[0],
            second_row_moved.images[1],
            second_row_moved.images[2],
        ],
    }
}

fn two_rows_three_one_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let first_row = three_columns_3_mosaic(first, second, third);
    let image4_dims = scale_width_dimension(fourth, first_row.total_size().width);

    MosaicImageDims {
        images: [
            first_row.images[0],
            first_row.images[1],
            first_row.images[2],
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: first_row.total_size().height + SPACING_SIZE,
                },
                dimensions: image4_dims,
                original_dimensions: fourth,
            },
        ],
    }
}

fn two_columns_of_two_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let first_col = top_bottom_2_mosaic(first, second);
    let second_col = top_bottom_2_mosaic(third, fourth);
    let scale_factor = second_col.total_size().height as f32 / first_col.total_size().height as f32;
    let second_col_moved = second_col.scale(scale_factor).add_width(first_col.total_size().width + SPACING_SIZE);

    MosaicImageDims {
        images: [
            first_col.images[0],
            first_col.images[1],
            second_col_moved.images[0],
            second_col_moved.images[1],
        ],
    }
}

fn two_columns_one_three_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let second_col = three_rows_3_mosaic(second, third, fourth);
    let image1_dims = scale_height_dimension(first, second_col.total_size().height);
    let second_col_moved = second_col.add_width(image1_dims.width + SPACING_SIZE);

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: image1_dims,
                original_dimensions: first,
            },
            second_col_moved.images[0],
            second_col_moved.images[1],
            second_col_moved.images[2],
        ],
    }
}

fn two_columns_three_one_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let first_col = three_rows_3_mosaic(first, second, third);
    let image4_dims = scale_height_dimension(fourth, first_col.total_size().height);

    MosaicImageDims {
        images: [
            first_col.images[0],
            first_col.images[1],
            first_col.images[2],
            ImageOffset {
                offset: Size {
                    width: first_col.total_size().width + SPACING_SIZE,
                    height: 0,
                },
                dimensions: image4_dims,
                original_dimensions: fourth,
            },
        ],
    }
}

fn three_rows_211_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let first_row = left_right_2_mosaic(first, second);
    let image3_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: first_row.total_size().height + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(third, first_row.total_size().width),
        original_dimensions: third,
    };

    MosaicImageDims {
        images: [
            first_row.images[0],
            first_row.images[1],
            image3_offset,
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: image3_offset.total_height() + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(fourth, first_row.total_size().width),
                original_dimensions: fourth,
            }
        ],
    }
}

fn three_rows_121_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let second_row = left_right_2_mosaic(second, third);
    let image1_dims = scale_width_dimension(first, second_row.total_size().width);
    let second_row_moved = second_row.add_height(image1_dims.height + SPACING_SIZE);

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: image1_dims,
                original_dimensions: first,
            },
            second_row_moved.images[0],
            second_row_moved.images[1],
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: second_row_moved.total_size().height + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(fourth, second_row_moved.total_size().width),
                original_dimensions: fourth,
            },
        ],
    }
}

fn three_rows_112_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let third_row = left_right_2_mosaic(third, fourth);
    let image1_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: 0,
        },
        dimensions: scale_width_dimension(first, third_row.total_size().width),
        original_dimensions: first,
    };
    let image2_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: image1_offset.total_height() + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(second, third_row.total_size().width),
        original_dimensions: second,
    };
    let third_row_moved = third_row.add_height(image2_offset.total_height() + SPACING_SIZE);

    MosaicImageDims {
        images: [
            image1_offset,
            image2_offset,
            third_row_moved.images[0],
            third_row_moved.images[1],
        ],
    }
}

fn three_columns_211_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let first_col = top_bottom_2_mosaic(first, second);
    let image3_offset = ImageOffset {
        offset: Size {
            width: first_col.total_size().width + SPACING_SIZE,
            height: 0,
        },
        dimensions: scale_height_dimension(third, first_col.total_size().height),
        original_dimensions: third,
    };

    MosaicImageDims {
        images: [
            first_col.images[0],
            first_col.images[1],
            image3_offset,
            ImageOffset {
                offset: Size {
                    width: image3_offset.total_width() + SPACING_SIZE,
                    height: 0,
                },
                dimensions: scale_height_dimension(fourth, first_col.total_size().height),
                original_dimensions: fourth,
            },
        ],
    }
}

fn three_columns_121_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let second_col = top_bottom_2_mosaic(second, third);
    let image1_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: 0,
        },
        dimensions: scale_height_dimension(first, second_col.total_size().height),
        original_dimensions: first,
    };
    let second_col_moved = second_col.add_width(image1_offset.total_width() + SPACING_SIZE);

    MosaicImageDims {
        images: [
            image1_offset,
            second_col_moved.images[0],
            second_col_moved.images[1],
            ImageOffset {
                offset: Size {
                    width: second_col_moved.total_size().width + SPACING_SIZE,
                    height: 0,
                },
                dimensions: scale_height_dimension(fourth, second_col_moved.total_size().height),
                original_dimensions: fourth,
            },
        ],
    }
}

fn three_columns_112_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let third_col = top_bottom_2_mosaic(third, fourth);
    let image1_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: 0,
        },
        dimensions: scale_height_dimension(first, third_col.total_size().height),
        original_dimensions: first,
    };
    let image2_offset = ImageOffset {
        offset: Size {
            width: image1_offset.total_width() + SPACING_SIZE,
            height: 0,
        },
        dimensions: scale_height_dimension(second, third_col.total_size().height),
        original_dimensions: second,
    };
    let third_col_moved = third_col.add_width(image2_offset.total_width() + SPACING_SIZE);

    MosaicImageDims {
        images: [
            image1_offset,
            image2_offset,
            third_col_moved.images[0],
            third_col_moved.images[1],
        ],
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use image::{Rgb, RgbImage};
    use crate::mosaic;

    const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
    const RED: Rgb<u8> = Rgb([255, 0, 0]);
    const BLUE: Rgb<u8> = Rgb([0, 0, 255]);
    const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
    const PURPLE: Rgb<u8> = Rgb([255, 64, 255]);
    const TEST_RESULT_DIR: &str = "./mosaic_tests/";

    fn create_with_colour(width: u32, height: u32, colour: Rgb<u8>) -> RgbImage {
        let mut img = RgbImage::new(width, height);

        for x in 0..width {
            for y in 0..height {
                img.put_pixel(x, y, colour);
                img.put_pixel(x, y, colour);
            }
        }

        img
    }

    fn is_colour_at_pixel(x: u32, y: u32, image: &RgbImage, colour: Rgb<u8>) -> bool {
        image.get_pixel(x, y).eq(&colour)
    }

    fn is_colour_in_range(start_x: u32, start_y: u32, end_x: u32, end_y: u32, image: &RgbImage, colour: Rgb<u8>) -> bool {
        for x in start_x..end_x {
            for y in start_y..end_y {
                if !is_colour_at_pixel(x, y, image, colour) {
                    return false;
                }
            }
        }
        true
    }

    fn has_black_vertical_line(x: u32, image: &RgbImage) -> bool {
        is_colour_in_range(x, 0, x, image.height(), image, BLACK)
    }

    fn has_black_horizontal_line(y: u32, image: &RgbImage) -> bool {
        is_colour_in_range(0, y, image.width(), y, image, BLACK)
    }

    fn has_black_vertical_line_partial(x: u32, start_y: u32, end_y: u32, image: &RgbImage) -> bool {
        is_colour_in_range(x, start_y, x, end_y, image, BLACK)
    }

    fn has_black_horizontal_line_partial(y: u32, start_x: u32, end_x: u32, image: &RgbImage) -> bool {
        is_colour_in_range(start_x, y, end_x, y, image, BLACK)
    }

    fn save_result(result: &RgbImage, filename: &str) {
        let file_path = [TEST_RESULT_DIR, filename, ".png"].join("");
        fs::create_dir_all(TEST_RESULT_DIR).unwrap();
        result.save(file_path).unwrap();
    }

    #[test]
    fn mosaic_2_left_right() {
        let left = create_with_colour(100, 400, RED);
        let right = create_with_colour(200, 400, BLUE);

        let result = mosaic(vec![left, right]);

        save_result(&result, "2-left_right");
        assert!(is_colour_in_range(0, 0, 100, 400, &result, RED));
        assert!(is_colour_in_range(120, 0, 300, 400, &result, BLUE));
        assert!(has_black_vertical_line(105, &result));
    }

    #[test]
    fn mosaic_2_top_bottom() {
        let top = create_with_colour(400, 200, RED);
        let bottom = create_with_colour(400, 100, BLUE);

        let result = mosaic(vec![top, bottom]);

        save_result(&result, "2-top_bottom");
        assert!(is_colour_in_range(0, 0, 400, 200, &result, RED));
        assert!(is_colour_in_range(0, 220, 400, 300, &result, BLUE));
        assert!(has_black_horizontal_line(205, &result));
    }

    #[test]
    fn mosaic_3_three_cols() {
        let left = create_with_colour(100, 400, RED);
        let mid = create_with_colour(200, 400, BLUE);
        let right = create_with_colour(100, 400, GREEN);

        let result = mosaic(vec![left, mid, right]);

        save_result(&result, "3-three_cols");
        assert!(is_colour_in_range(0, 0, 100, 400, &result, RED));
        assert!(has_black_vertical_line(105, &result));
        assert!(is_colour_in_range(120, 0,300, 400, &result, BLUE));
        assert!(has_black_vertical_line(315, &result));
        assert!(is_colour_in_range(330, 0, 400, 400, &result, GREEN));
    }

    #[test]
    fn mosaic_3_top_top_bottom() {
        let top_left = create_with_colour(200, 300, RED);
        let top_right = create_with_colour(200, 300, BLUE);
        let bottom = create_with_colour(400, 100, GREEN);

        let result = mosaic(vec![top_left, top_right, bottom]);

        save_result(&result, "3-top_top_bottom");
        assert!(is_colour_in_range(0, 0, 200, 300, &result, RED));
        assert!(has_black_vertical_line_partial(205, 0, 300, &result));
        assert!(is_colour_in_range(220, 0, 400, 300, &result, BLUE));
        assert!(has_black_horizontal_line(305, &result));
        assert!(is_colour_in_range(0, 320, 400, 400, &result, GREEN));
    }

    #[test]
    fn mosaic_3_left_left_right() {
        let left_top = create_with_colour(300, 200, RED);
        let left_bot = create_with_colour(300, 200, BLUE);
        let right = create_with_colour(100, 400, GREEN);

        let result = mosaic(vec![left_top, left_bot, right]);

        save_result(&result, "3-left_left_right");
        assert!(is_colour_in_range(0, 0, 300, 200, &result, RED));
        assert!(has_black_horizontal_line_partial(205, 0, 300, &result));
        assert!(is_colour_in_range(0, 220, 300, 400, &result, BLUE));
        assert!(has_black_vertical_line(305, &result));
        assert!(is_colour_in_range(320, 0, 400, 400, &result, GREEN));
    }

    #[test]
    fn mosaic_3_left_right_right() {
        let left = create_with_colour(100, 400, RED);
        let right_top = create_with_colour(300, 200, BLUE);
        let right_bot = create_with_colour(300, 200, GREEN);

        let result = mosaic(vec![left, right_top, right_bot]);

        save_result(&result, "3-left_right_right");
        assert!(is_colour_in_range(0, 0, 100, 400, &result, RED));
        assert!(has_black_vertical_line(105, &result));
        assert!(is_colour_in_range(120, 0, 400, 200, &result, BLUE));
        assert!(has_black_horizontal_line_partial(205, 120, 400, &result));
        assert!(is_colour_in_range(120, 220, 400, 400, &result, GREEN));
    }

    #[test]
    fn mosaic_3_top_bottom_bottom() {
        let top = create_with_colour(400, 100, RED);
        let bot_left = create_with_colour(200, 300, BLUE);
        let bot_right = create_with_colour(200, 300, GREEN);

        let result = mosaic(vec![top, bot_left, bot_right]);

        save_result(&result, "3-top_bottom_bottom");
        assert!(is_colour_in_range(0, 0, 400, 100, &result, RED));
        assert!(has_black_horizontal_line(105, &result));
        assert!(is_colour_in_range(0, 120, 200, 400, &result, BLUE));
        assert!(has_black_vertical_line_partial(205, 120, 400, &result));
        assert!(is_colour_in_range(220, 120, 400, 400, &result, GREEN));
    }

    #[test]
    fn mosaic_3_three_rows() {
        let row1 = create_with_colour(300, 100, RED);
        let row2 = create_with_colour(300, 100, BLUE);
        let row3 = create_with_colour(300, 100, GREEN);

        let result = mosaic(vec![row1, row2, row3]);

        save_result(&result, "3-three_rows");
        assert!(is_colour_in_range(0, 0, 300, 100, &result, RED));
        assert!(has_black_horizontal_line(105, &result));
        assert!(is_colour_in_range(0, 120, 300, 200, &result, BLUE));
        assert!(has_black_horizontal_line(215, &result));
        assert!(is_colour_in_range(0, 230, 300, 300, &result, GREEN));
    }

    #[test]
    fn mosaic_4_four_cols() {
        let col1 = create_with_colour(100, 400, RED);
        let col2 = create_with_colour(100, 400, BLUE);
        let col3 = create_with_colour(100, 400, GREEN);
        let col4 = create_with_colour(100, 400, PURPLE);

        let result = mosaic(vec![col1, col2, col3, col4]);

        save_result(&result, "4-four_cols");
        assert!(is_colour_in_range(0, 0, 100, 400, &result, RED));
        assert!(has_black_vertical_line(105, &result));
        assert!(is_colour_in_range(120, 0,200, 400, &result, BLUE));
        assert!(has_black_vertical_line(215, &result));
        assert!(is_colour_in_range(230, 0, 300, 400, &result, GREEN));
        assert!(has_black_vertical_line(325, &result));
        assert!(is_colour_in_range(340, 0, 400, 400, &result, PURPLE));
    }

    #[test]
    fn mosaic_4_four_rows() {
        let row1 = create_with_colour(400, 100, RED);
        let row2 = create_with_colour(400, 100, BLUE);
        let row3 = create_with_colour(400, 100, GREEN);
        let row4 = create_with_colour(400, 100, PURPLE);

        let result = mosaic(vec![row1, row2, row3, row4]);

        save_result(&result, "4-four_rows");
        assert!(is_colour_in_range(0, 0, 400, 100, &result, RED));
        assert!(has_black_horizontal_line(105, &result));
        assert!(is_colour_in_range(0,120, 400, 200, &result, BLUE));
        assert!(has_black_horizontal_line(215, &result));
        assert!(is_colour_in_range(0, 230,  400, 300, &result, GREEN));
        assert!(has_black_horizontal_line(325, &result));
        assert!(is_colour_in_range(0, 340, 400, 400, &result, PURPLE));
    }

    #[test]
    fn mosaic_4_two_rows_of_two() {
        let top_left = create_with_colour(100, 200, RED);
        let top_right = create_with_colour(300, 200, BLUE);
        let bot_left = create_with_colour(300, 200, GREEN);
        let bot_right = create_with_colour(100, 200, PURPLE);

        let result = mosaic(vec![top_left, top_right, bot_left, bot_right]);

        save_result(&result, "4-two_rows_of_two");
        assert!(is_colour_in_range(0, 0, 100, 200, &result, RED));
        assert!(has_black_vertical_line_partial(105, 0, 200, &result));
        assert!(is_colour_in_range(120, 0, 400, 200, &result, BLUE));
        assert!(has_black_horizontal_line(205, &result));
        assert!(is_colour_in_range(0, 220, 300, 400, &result, GREEN));
        assert!(has_black_vertical_line_partial(305, 220, 400, &result));
        assert!(is_colour_in_range(320, 220, 400, 400, &result, PURPLE));
    }

    #[test]
    fn mosaic_4_two_rows_one_three() {
        let top = create_with_colour(300, 200, RED);
        let bot_left = create_with_colour(100, 100, BLUE);
        let bot_mid = create_with_colour(100, 100, GREEN);
        let bot_right = create_with_colour(100, 100, PURPLE);

        let result = mosaic(vec![top, bot_left, bot_mid, bot_right]);

        save_result(&result, "4-two_rows_one_three");
        assert!(is_colour_in_range(0, 0, 300, 200, &result, RED));
        assert!(has_black_horizontal_line(205, &result));
        assert!(is_colour_in_range(0, 230, 100, 300, &result, BLUE));
        assert!(has_black_vertical_line_partial(105, 220, 300, &result));
        assert!(is_colour_in_range(120, 230, 200, 300, &result, GREEN));
        assert!(has_black_vertical_line_partial(215, 220, 300, &result));
        assert!(is_colour_in_range(230, 230, 300, 300, &result, PURPLE));
    }

    #[test]
    fn mosaic_4_two_rows_three_one() {
        let top_left = create_with_colour(100, 100, RED);
        let top_mid = create_with_colour(100, 100, BLUE);
        let top_right = create_with_colour(100, 100, GREEN);
        let bottom = create_with_colour(300, 200, PURPLE);

        let result = mosaic(vec![top_left, top_mid, top_right, bottom]);

        save_result(&result, "4-two_rows_three_one");
        assert!(is_colour_in_range(0, 0, 100, 100, &result, RED));
        assert!(has_black_vertical_line_partial(105, 0, 100, &result));
        assert!(is_colour_in_range(120, 0, 200, 100, &result, BLUE));
        assert!(has_black_vertical_line_partial(215, 0, 100, &result));
        assert!(is_colour_in_range(230, 0, 300, 100, &result, GREEN));
        assert!(has_black_horizontal_line(105, &result));
        assert!(is_colour_in_range(0, 120, 300, 300, &result, PURPLE));
    }

    #[test]
    fn mosaic_4_two_columns_one_three() {
        let left = create_with_colour(200, 300, RED);
        let right_top = create_with_colour(100, 100, BLUE);
        let right_mid = create_with_colour(100, 100, GREEN);
        let right_bot = create_with_colour(100, 100, PURPLE);

        let result = mosaic(vec![left, right_top, right_mid, right_bot]);

        save_result(&result, "4-two_columns_one_three");
        assert!(is_colour_in_range(0, 0, 200, 300, &result, RED));
        assert!(has_black_vertical_line(105, &result));
        assert!(is_colour_in_range(230, 0, 300, 100, &result, BLUE));
        assert!(has_black_horizontal_line_partial(105, 220, 300, &result));
        assert!(is_colour_in_range(230, 120, 300, 200, &result, GREEN));
        assert!(has_black_horizontal_line_partial(215, 220, 300, &result));
        assert!(is_colour_in_range(230, 230, 300, 300, &result, PURPLE));
    }

    #[test]
    fn mosaic_4_two_columns_three_one() {
        let left_top = create_with_colour(100, 100, RED);
        let left_mid = create_with_colour(100, 100, BLUE);
        let left_bot = create_with_colour(100, 100, GREEN);
        let right = create_with_colour(200, 300, PURPLE);

        let result = mosaic(vec![left_top, left_mid, left_bot, right]);

        save_result(&result, "4-two_columns_three_one");
        assert!(is_colour_in_range(0, 0, 100, 100, &result, RED));
        assert!(has_black_horizontal_line_partial(105, 0, 200, &result));
        assert!(is_colour_in_range(0, 120, 100, 200, &result, BLUE));
        assert!(has_black_horizontal_line_partial(215, 0, 200, &result));
        assert!(is_colour_in_range(0, 230, 100, 300, &result, GREEN));
        assert!(has_black_vertical_line(105, &result));
        assert!(is_colour_in_range(220, 230, 300, 300, &result, PURPLE));
    }

    #[test]
    fn mosaic_4_three_rows_211() {
        let top_left = create_with_colour(300, 200, RED);
        let top_right = create_with_colour(300, 200, BLUE);
        let mid = create_with_colour(600, 200, GREEN);
        let bot = create_with_colour(600, 200, PURPLE);

        let result = mosaic(vec![top_left, top_right, mid, bot]);

        save_result(&result, "4-three_rows_211");
        assert!(is_colour_in_range(0, 0, 300, 200, &result, RED));
        assert!(has_black_vertical_line_partial(305, 0, 200, &result));
        assert!(is_colour_in_range(320, 0, 600, 200, &result, BLUE));
        assert!(has_black_horizontal_line(205, &result));
        assert!(is_colour_in_range(0, 220, 600, 400, &result, GREEN));
        assert!(has_black_horizontal_line(415, &result));
        assert!(is_colour_in_range(0, 430, 600, 600, &result, PURPLE));
    }

    #[test]
    fn mosaic_4_three_rows_121() {
        let top = create_with_colour(600, 200, RED);
        let mid_left = create_with_colour(300, 200, BLUE);
        let mid_right = create_with_colour(300, 200, GREEN);
        let bot = create_with_colour(600, 200, PURPLE);

        let result = mosaic(vec![top, mid_left, mid_right, bot]);

        save_result(&result, "4-three_rows_121");
        assert!(is_colour_in_range(0, 0, 600, 200, &result, RED));
        assert!(has_black_horizontal_line(205, &result));
        assert!(is_colour_in_range(0, 220, 300, 400, &result, BLUE));
        assert!(has_black_vertical_line_partial(305, 210, 400, &result));
        assert!(is_colour_in_range(320, 220, 600, 400, &result, GREEN));
        assert!(has_black_horizontal_line(415, &result));
        assert!(is_colour_in_range(0, 430, 600, 600, &result, PURPLE));
    }

    #[test]
    fn mosaic_4_three_rows_112() {
        let top = create_with_colour(600, 200, RED);
        let mid = create_with_colour(600, 200, BLUE);
        let bot_left = create_with_colour(300, 200, GREEN);
        let bot_right = create_with_colour(300, 200, PURPLE);

        let result = mosaic(vec![top, mid, bot_left, bot_right]);

        save_result(&result, "4-three_rows_112");
        assert!(is_colour_in_range(0, 0, 600, 200, &result, RED));
        assert!(has_black_horizontal_line(205, &result));
        assert!(is_colour_in_range(0, 220, 600, 400, &result, BLUE));
        assert!(has_black_horizontal_line(415, &result));
        assert!(is_colour_in_range(0, 430, 300, 600, &result, GREEN));
        assert!(has_black_vertical_line_partial(305, 420, 600, &result));
        assert!(is_colour_in_range(320, 430, 600, 600, &result, PURPLE));
    }

    #[test]
    fn pick_less_square_option_for_better_scaling_ratio() {
        let top_left = create_with_colour(100, 100, RED);
        let top_right = create_with_colour(300, 100, BLUE);
        let bot_left = create_with_colour(300, 100, GREEN);
        let bot_right = create_with_colour(100, 100, PURPLE);

        let result = mosaic(vec![top_left, top_right, bot_left, bot_right]);

        save_result(&result, "less_square_better_scaling_ratio");
        assert!(is_colour_in_range(0, 0, 100, 100, &result, RED));
        assert!(has_black_vertical_line_partial(105, 0, 100, &result));
        assert!(is_colour_in_range(120, 0, 400, 100, &result, BLUE));
        assert!(has_black_horizontal_line(105, &result));
        assert!(is_colour_in_range(0, 120, 300, 200, &result, GREEN));
        assert!(has_black_vertical_line_partial(305, 120, 200, &result));
        assert!(is_colour_in_range(320, 120, 400, 200, &result, PURPLE));
    }

    #[test]
    fn wont_scale_down_to_match() {
        let left = create_with_colour(100, 200, RED);
        let right = create_with_colour(200, 400, BLUE);

        let result = mosaic(vec![left, right]);

        save_result(&result, "wont_scale_down_to_match");
        assert!(is_colour_in_range(0, 0, 200, 400, &result, RED));
        assert!(has_black_vertical_line(205, &result));
        assert!(is_colour_in_range(220, 0, 400, 400, &result, BLUE));
    }

    #[test]
    fn scale_down_to_fit() {
        let left = create_with_colour(3000, 3300, RED);
        let right = create_with_colour(3000, 3300, BLUE);

        let result = mosaic(vec![left, right]);

        save_result(&result, "scale_down_to_fit");
        assert!(is_colour_in_range(0, 0, 1980, 2180, &result, RED));
        assert!(has_black_vertical_line(2000, &result));
        assert!(is_colour_in_range(2020, 0, 4000, 2180, &result, BLUE));
    }
}
