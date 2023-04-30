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
