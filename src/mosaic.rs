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
use std::cmp::Ordering::Equal;
use std::iter::zip;

use image::{imageops::FilterType, RgbImage};
use tracing::instrument;

const SPACING_SIZE: u32 = 10;
const MAX_SIZE: u32 = 4000;

pub fn mosaic(mut images: VecDeque<RgbImage>) -> RgbImage {
    match images.len() {
        2 => {
            let first = images.pop_front().unwrap();
            let second = images.pop_front().unwrap();
            build_2_mosaic(first, second)
        }
        3 => {
            let first = images.pop_front().unwrap();
            let second = images.pop_front().unwrap();
            let third = images.pop_front().unwrap();
            build_3_mosaic(first, second, third)
        }
        4 => {
            let first = images.pop_front().unwrap();
            let second = images.pop_front().unwrap();
            let third = images.pop_front().unwrap();
            let fourth = images.pop_front().unwrap();
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
            dimensions: Size {
                width: self.dimensions.width,
                height: self.dimensions.height,
            },
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
    let total_size = mosaic.total_size();
    let scale_factor = overall_scale_factor(total_size);
    let scaled_mosaic = mosaic.scale(scale_factor);

    let resize_args = zip(images, scaled_mosaic.images).map(|(image, offset)| {
        (
            image,
            offset.dimensions,
        )
    }).collect();

    let resized = resize_images(resize_args);

    let mut background = create_background(scaled_mosaic.total_size());
    for (image, offset) in zip(resized, scaled_mosaic.images) {
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
            },
            ImageOffset {
                offset: Size {
                    width: first.width + SPACING_SIZE,
                    height: 0,
                },
                dimensions: scale_height_dimension(second, first.height),
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
            },
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: first.height + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(second, first.width),
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
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
            },
            image2_offset,
            ImageOffset {
                offset: Size {
                    width: image2_offset.total_width() + SPACING_SIZE,
                    height: 0
                },
                dimensions: scale_height_dimension(third, first.height),
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
        dimensions: scale_height_dimension(second, first.height)
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
            },
            image2_offset,
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: first.height + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(third, image2_offset.total_width()),
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
        dimensions: scale_width_dimension(second, first.width)
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
            },
            image2_offset,
            ImageOffset {
                offset: Size {
                    width: first.width + SPACING_SIZE,
                    height: 0,
                },
                dimensions: scale_height_dimension(third, image2_offset.total_height()),
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
    };
    let image3_offset = ImageOffset {
        offset: Size {
            width: image1_dims.width + SPACING_SIZE,
            height: image2_offset.total_height() + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(third, second.width),
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: image1_dims,
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
            },
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: image1_dims.height + SPACING_SIZE,
                },
                dimensions: second,
            },
            ImageOffset {
                offset: Size {
                    width: second.width + SPACING_SIZE,
                    height: image1_dims.height + SPACING_SIZE,
                },
                dimensions: image3_dims,
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
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
            },
            image2_offset,
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: image2_offset.total_height() + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(third, first.width),
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
    };
    let image3_offset = ImageOffset {
        offset: Size {
            width: image2_offset.total_width() + SPACING_SIZE,
            height: 0,
        },
        dimensions: scale_height_dimension(third, first.height),
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
            },
            image2_offset,
            image3_offset,
            ImageOffset {
                offset: Size {
                    width: image3_offset.total_width() + SPACING_SIZE,
                    height: 0,
                },
                dimensions: scale_height_dimension(fourth, first.height),
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
    };
    let image3_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: image2_offset.total_height() + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(third, first.width)
    };

    MosaicImageDims {
        images: [
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: 0,
                },
                dimensions: first,
            },
            image2_offset,
            image3_offset,
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: image3_offset.total_height() + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(fourth, first.width),
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
    };

    MosaicImageDims {
        images: [
            first_row.images[0],
            first_row.images[1],
            image3_offset,
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: image3_offset.total_height(),
                },
                dimensions: scale_width_dimension(fourth, first_row.total_size().width),
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
            },
            second_row_moved.images[0],
            second_row_moved.images[1],
            ImageOffset {
                offset: Size {
                    width: 0,
                    height: second_row_moved.total_size().height + SPACING_SIZE,
                },
                dimensions: scale_width_dimension(fourth, second_row_moved.total_size().width),
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
    };
    let image2_offset = ImageOffset {
        offset: Size {
            width: 0,
            height: image1_offset.total_height() + SPACING_SIZE,
        },
        dimensions: scale_width_dimension(second, third_row.total_size().width),
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
    };
    let image2_offset = ImageOffset {
        offset: Size {
            width: image1_offset.total_width() + SPACING_SIZE,
            height: 0,
        },
        dimensions: scale_height_dimension(second, third_col.total_size().height),
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
