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

pub fn build_2_mosaic(first: RgbImage, second: RgbImage) -> RgbImage {
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

fn best_2_mosaic(first: Size, second: Size) -> MosaicImageDims<2> {
    let top_bottom = top_bottom_2_mosaic(first, second);
    let left_right = left_right_2_mosaic(first, second);
    return best_mosaic(&[&top_bottom, &left_right]);
}

pub fn left_right_2_mosaic(first: Size, second: Size) -> MosaicImageDims<2> {
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

pub fn top_bottom_2_mosaic(first: Size, second: Size) -> MosaicImageDims<2> {
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

#[cfg(test)]
mod tests {
    use crate::mosaic;
    use crate::mosaic::testutils::{
        BLUE,
        create_with_colour,
        has_black_horizontal_line,
        has_black_vertical_line,
        is_colour_in_range,
        RED,
        save_result,
    };

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
}