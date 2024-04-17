use image::RgbImage;

use crate::distance::EuclideanDistanceMap;

pub struct GlobalData<'a> {
    pub rgb_image: &'a RgbImage,

    pub euclidean_distance_map: &'a EuclideanDistanceMap,

    pub width: usize,

    pub height: usize,
}
