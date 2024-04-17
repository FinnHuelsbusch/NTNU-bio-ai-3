use std::env;

use config::{ initialize_config, Config };

use crate::{
    distance::calculate_euclidean_distance_map_for_neighbors,
    genetic_algorithm::run_genetic_algorithm_instance,
    global_data::{ GlobalData },
    individual::Individual,
    population::non_dominated_sort,
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

    let rgb_image = Individual::open_image_as_rgb(&config.picture_path);

    let global_data = GlobalData {
        rgb_image: &rgb_image,
        euclidean_distance_map: &calculate_euclidean_distance_map_for_neighbors(&rgb_image),

        width: rgb_image.width() as usize,
        height: rgb_image.height() as usize,
    };
    run_genetic_algorithm_instance(&config, &global_data);
}
