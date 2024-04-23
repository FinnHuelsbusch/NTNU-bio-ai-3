use std::env;

use config::{ initialize_config, Config };
use image::{ imageops::blur, GrayImage, ImageBuffer, Luma, Pixel, RgbImage };
use imageproc::filter::gaussian_blur_f32;
use rand::{ thread_rng, Rng };

use crate::{
    distance::calculate_euclidean_distance_map_for_neighbors,
    genetic_algorithm::run_genetic_algorithm_instance,
    global_data::{ generate_pixel_edge_weights, GlobalData },
    individual::Individual,
    utils::show,
};

mod config;
mod crossover_functions;
mod distance;
mod genetic_algorithm;
mod individual;
mod mutation_functions;
mod population;
mod selection_functions;
mod utils;
mod global_data;

/**
 * We do create a lot of data here to use as global data later. It is all done at the start of the main function to ensure a global lifetime in the root closure
 */
#[show_image::main]
fn main() {
    let args: Vec<String> = env::args().collect();
    let config_path: &str;
    if args.len() < 2 {
        config_path = "./config.json";
    } else {
        config_path = &args[1];
    }
    // Load config
    let config: Config = initialize_config(config_path);
    println!("{}", serde_json::to_string_pretty(&config).unwrap());

    // Load the rgb image for the global data
    let rgb_image = Individual::open_image_as_rgb(
        &format!("./Project 3 training_images/{}/Test image.jpg", config.problem_instance)
    );

    // Load the edge image and the biased weights
    let edge_image = Individual::open_image_as_edge_map(
        &format!("./Project 3 training_images/{}/Test image.jpg", config.problem_instance),
        config.canny_hysteresis_low,
        config.canny_hysteresis_high
    );
    // Blur the image so we have a bit of a buffer around the edges
    let mut blurred = gaussian_blur_f32(&edge_image, config.blur_sigma);

    // add delta to each pixel so we dont loose mutations at spots which are not edges
    blurred.pixels_mut().for_each(|pixel| {
        *pixel = Luma([pixel.0[0] + 1]);
    });

    // Clip the values between 0 and 255
    blurred.pixels_mut().for_each(|pixel| {
        *pixel = Luma([pixel.0[0].clamp(0u8, 255u8)]);
    });

    // Scale the image between 0 and 255 (assuming the original values were not already scaled)
    let max_val = blurred.iter().cloned().max().unwrap();
    let scale = 255.0 / (max_val as f32);
    blurred.pixels_mut().for_each(|pixel| {
        *pixel = Luma([((pixel.0[0] as f32) * scale).round() as u8]);
    });

    let weights = generate_pixel_edge_weights(&blurred);

    // let mut rgb_image_edge: RgbImage = ImageBuffer::new(edge_image.width(), edge_image.height());

    // for row in 0..blurred.height() {
    //     for column in 0..blurred.width() {
    //         let pixel = blurred.get_pixel(column as u32, row as u32);
    //         let copy_pixel = rgb_image_edge.get_pixel_mut(column as u32, row as u32);
    //         *copy_pixel = image::Rgb([pixel.0[0], pixel.0[0], pixel.0[0]]);
    //     }
    // }

    // show(&rgb_image_edge);

    let global_data = GlobalData {
        rgb_image: &rgb_image,
        euclidean_distance_map: &calculate_euclidean_distance_map_for_neighbors(&rgb_image),
        edge_image: &edge_image,
        width: rgb_image.width() as usize,
        height: rgb_image.height() as usize,
        pixel_weights: &weights,
    };
    run_genetic_algorithm_instance(&config, &global_data);
}
