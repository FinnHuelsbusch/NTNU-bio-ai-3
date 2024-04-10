use std::env;

use config::{ initialize_config, Config };

use crate::genetic_algorithm::run_genetic_algorithm_instance;

mod individual;
mod config;
mod genetic_algorithm;
mod population;
mod selection_functions;
mod crossover_functions;
mod mutation_functions;
mod utils;

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
    run_genetic_algorithm_instance(&config);
}
