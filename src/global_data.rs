use image::RgbImage;

use crate::{
    config::Config,
    distance::{ calculate_euclidean_distance_map_for_neighbors, EuclideanDistanceMap },
    individual::Individual,
};

pub struct GlobalData<'a> {
    pub rgb_image: &'a RgbImage,

    pub euclidean_distance_map: &'a EuclideanDistanceMap,

    pub width: usize,

    pub height: usize,
}
