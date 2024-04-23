use image::{ GrayImage, RgbImage };

use crate::distance::EuclideanDistanceMap;

pub struct GlobalData<'a> {
    pub rgb_image: &'a RgbImage,
    pub edge_image: &'a GrayImage,
    pub pixel_weights: &'a Vec<f64>,

    pub euclidean_distance_map: &'a EuclideanDistanceMap,

    pub width: usize,

    pub height: usize,
}

pub fn generate_pixel_edge_weights(image: &GrayImage) -> Vec<f64> {
    let (width, height) = image.dimensions();
    let mut weights = vec![0.0; (width * height) as usize];
    let mut total_luminance = 0.0;

    // Calculate weights based on pixel luminance
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let luminance = pixel[0] as f64; // Assuming pixel data is u8
            weights[(y * width + x) as usize] = luminance;
            total_luminance += luminance;
        }
    }

    // Normalize weights to create a probability distribution
    for weight in weights.iter_mut() {
        *weight /= total_luminance;
    }

    weights
}
