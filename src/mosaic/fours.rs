use image::RgbImage;

use crate::mosaic::{best_mosaic, build_mosaic, ImageOffset, MosaicDims, MosaicImageDims, scale_height_dimension, scale_width_dimension, Size, SPACING_SIZE};
use crate::mosaic::threes::{three_columns_3_mosaic, three_rows_3_mosaic};
use crate::mosaic::twos::{left_right_2_mosaic, top_bottom_2_mosaic};

pub fn build_4_mosaic(first: RgbImage, second: RgbImage, third: RgbImage, fourth: RgbImage) -> RgbImage {
    let first_size = Size { width: first.width(), height: first.height() };
    let second_size = Size { width: second.width(), height: second.height() };
    let third_size = Size { width: third.width(), height: third.height() };
    let fourth_size = Size { width: fourth.width(), height: fourth.height() };
    let best_mosaic = best_4_mosaic(first_size, second_size, third_size, fourth_size);
    build_mosaic(best_mosaic, [first, second, third, fourth])
}

fn best_4_mosaic(first: Size, second: Size, third: Size, fourth: Size) -> MosaicImageDims<4> {
    let four_columns = four_columns_4_mosaic(first, second, third, fourth);
    let four_rows = four_rows_4_mosaic(first, second, third, fourth);
    let two_rows_of_two = two_rows_of_two_4_mosaic(first, second, third, fourth);
    let two_rows_one_three = two_rows_one_three_4_mosaic(first, second, third, fourth);
    let two_rows_three_one = two_rows_three_one_4_mosaic(first, second, third, fourth);
    let two_columns_one_three = two_columns_one_three_4_mosaic(first, second, third, fourth);
    let two_columns_three_one = two_columns_three_one_4_mosaic(first, second, third, fourth);
    let three_rows_211 = three_rows_211_4_mosaic(first, second, third, fourth);
    let three_rows_121 = three_rows_121_4_mosaic(first, second, third, fourth);
    let three_rows_112 = three_rows_112_4_mosaic(first, second, third, fourth);
    // These four are omitted from the options, as they are just not very readable
    // let two_columns_of_two = two_columns_of_two_4_mosaic(first, second, third, fourth);
    // let three_columns_211 = three_columns_211_4_mosaic(first, second, third, fourth);
    // let three_columns_121 = three_columns_121_4_mosaic(first, second, third, fourth);
    // let three_columns_112 = three_columns_112_4_mosaic(first, second, third, fourth);
    return best_mosaic(&[
        &four_columns,
        &four_rows,
        &two_rows_of_two,
        &two_rows_one_three,
        &two_rows_three_one,
        &two_columns_one_three,
        &two_columns_three_one,
        &three_rows_211,
        &three_rows_121,
        &three_rows_112
    ]);
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
        PURPLE,
        RED,
        save_result,
    };

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
        assert!(is_colour_in_range(120, 0, 200, 400, &result, BLUE));
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
        assert!(is_colour_in_range(0, 120, 400, 200, &result, BLUE));
        assert!(has_black_horizontal_line(215, &result));
        assert!(is_colour_in_range(0, 230, 400, 300, &result, GREEN));
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
}