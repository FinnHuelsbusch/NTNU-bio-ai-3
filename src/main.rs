use std::env;

use config::{ initialize_config, Config };

use crate::{genetic_algorithm::run_genetic_algorithm_instance, individual::Individual, population::non_dominated_sort};

mod individual;
mod config;
mod genetic_algorithm;
mod population;
mod selection_functions;
mod crossover_functions;
mod mutation_functions;
mod utils;
mod distance;

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

// let mut population: Vec<Individual> = Vec::new();
// let mut individual1 = Individual::new(&config.picture_path);
// let mut individual2 = Individual::new(&config.picture_path);
// // let mut individual3 = Individual::new(&config.picture_path);
// // let mut individual4 = Individual::new(&config.picture_path);

// individual1.edge_value_fitness = 64898.847619;
// individual1.connectivity_fitness = 623044.687349;
// individual1.overall_deviation_fitness = 5.552130e+06;

// individual2.edge_value_fitness = 64898.847619;
// individual2.connectivity_fitness = 623044.687349;
// individual2.overall_deviation_fitness = 5.552130e+06;

// individual3.edge_value_fitness = 5566618.5785547365;
// individual3.connectivity_fitness = 65155.629761843593;
// individual3.overall_deviation_fitness = 626991.80390100495;

// individual4.edge_value_fitness = 5637881.0635476178;
// individual4.connectivity_fitness = 65653.591666603563;
// individual4.overall_deviation_fitness = 615365.04962341569;

// population.push(individual1);
// population.push(individual2);
// population.push(individual3);
// population.push(individual4);

// let current_population_ranked = non_dominated_sort(&population);
// print!("{:?}", current_population_ranked[0]); 

// println!("{:?}", population[0].dominates(&population[1]));
// println!("{:?}", population[1].dominates(&population[0]));

// // print non dominated sort
// let current_population_ranked = non_dominated_sort(&population);
// for i in 0..current_population_ranked.len() {
//     println!("Frontier {}", i);
//     for j in 0..current_population_ranked[i].len() {
//         println!("{:?}", current_population_ranked[i][j]);
//     }

// }
}
