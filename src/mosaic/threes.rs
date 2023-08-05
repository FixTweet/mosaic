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


#[cfg(test)]
mod tests {
    use crate::mosaic;
    use crate::mosaic::testutils::{
        BLUE,
        create_with_colour,
        GREEN,
        has_black_horizontal_line,
        has_black_horizontal_line_partial,
        has_black_vertical_line,
        has_black_vertical_line_partial,
        is_colour_in_range,
        RED,
        save_result,
    };

    #[test]
    fn mosaic_3_three_cols() {
        let left = create_with_colour(100, 400, RED);
        let mid = create_with_colour(200, 400, BLUE);
        let right = create_with_colour(100, 400, GREEN);

        let result = mosaic(vec![left, mid, right]);

        save_result(&result, "3-three_cols");
        assert!(is_colour_in_range(0, 0, 100, 400, &result, RED));
        assert!(has_black_vertical_line(105, &result));
        assert!(is_colour_in_range(120, 0, 300, 400, &result, BLUE));
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
}