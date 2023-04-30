use image::RgbImage;

use crate::mosaic::{
    best_mosaic,
    build_mosaic,
    ImageOffset,
    MosaicImageDims,
    scale_height_dimension,
    scale_width_dimension,
    Size,
    SPACING_SIZE,
};

pub fn build_3_mosaic(first: RgbImage, second: RgbImage, third: RgbImage) -> RgbImage {
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

fn best_3_mosaic(first: Size, second: Size, third: Size) -> MosaicImageDims<3> {
    let three_columns = three_columns_3_mosaic(first, second, third);
    let top_top_bottom = top_top_bottom_3_mosaic(first, second, third);
    let left_right_right = left_right_right_3_mosaic(first, second, third);
    let left_left_right = left_left_right_3_mosaic(first, second, third);
    let top_bottom_bottom = top_bottom_bottom_3_mosaic(first, second, third);
    let three_rows = three_rows_3_mosaic(first, second, third);
    return best_mosaic(&[&three_columns, &top_top_bottom, &left_left_right, &left_right_right, &top_bottom_bottom, &three_rows]);
}

pub fn three_columns_3_mosaic(first: Size, second: Size, third: Size) -> MosaicImageDims<3> {
    let image2_offset = ImageOffset {
        offset: Size {
            width: first.width + SPACING_SIZE,
            height: 0,
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
                    height: 0,
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
            height: 0,
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

pub fn three_rows_3_mosaic(first: Size, second: Size, third: Size) -> MosaicImageDims<3> {
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
