use std::i128::MIN;
use std::io::{self, Write};

use crate::crossover_functions::crossover;
use crate::individual;
use crate::mutation_functions::mutate;
use crate::selection_functions::{parent_selection, survivor_selection};
use crate::{config::Config, population::Population};
use crate::population::{initialize_random_population, non_dominated_sort};


fn log_population_statistics(population: &Population, current_population_ranked: &Vec<Vec<usize>>) {
    // number of individuals in the skyline
    println!("Skyline: {:?}", current_population_ranked[0].len());
    // statistics of the skyline
    let mut min_edge_value_fitness = f64::MAX;
    let mut max_edge_value_fitness = f64::MIN;
    let mut avg_edge_value_fitness = 0.0;
    let mut min_connectivity_fitness = f64::MAX;
    let mut max_connectivity_fitness = f64::MIN;
    let mut avg_connectivity_fitness = 0.0;
    let mut min_overall_deviation_fitness = f64::MAX;
    let mut max_overall_deviation_fitness = f64::MIN;
    let mut avg_overall_deviation_fitness = 0.0;

    for individual in current_population_ranked[0].iter() {
        let edge_value_fitness = population[*individual].edge_value_fitness;
        let connectivity_fitness = population[*individual].connectivity_fitness;
        let overall_deviation_fitness = population[*individual].overall_deviation_fitness;

        if edge_value_fitness < min_edge_value_fitness {
            min_edge_value_fitness = edge_value_fitness;
        }
        if edge_value_fitness > max_edge_value_fitness {
            max_edge_value_fitness = edge_value_fitness;
        }
        avg_edge_value_fitness += edge_value_fitness;

        if connectivity_fitness < min_connectivity_fitness {
            min_connectivity_fitness = connectivity_fitness;
        }
        if connectivity_fitness > max_connectivity_fitness {
            max_connectivity_fitness = connectivity_fitness;
        }
        avg_connectivity_fitness += connectivity_fitness;

        if overall_deviation_fitness < min_overall_deviation_fitness {
            min_overall_deviation_fitness = overall_deviation_fitness;
        }
        if overall_deviation_fitness > max_overall_deviation_fitness {
            max_overall_deviation_fitness = overall_deviation_fitness;
        }
        avg_overall_deviation_fitness += overall_deviation_fitness;
    }

    avg_edge_value_fitness /= current_population_ranked[0].len() as f64;
    avg_connectivity_fitness /= current_population_ranked[0].len() as f64;
    avg_overall_deviation_fitness /= current_population_ranked[0].len() as f64;

    // print as table
    println!(
        "Statistics: | Edge Value Fitness | Connectivity Fitness | Overall Deviation Fitness"
    );
    println!(
        "Min:        | {:<18.2} | {:<19.2} | {:<24.2}",
        min_edge_value_fitness, min_connectivity_fitness, min_overall_deviation_fitness
    );
    println!(
        "Max:        | {:<18.2} | {:<19.2} | {:<24.2}",
        max_edge_value_fitness, max_connectivity_fitness, max_overall_deviation_fitness
    );
    println!(
        "Avg:        | {:<18.2} | {:<19.2} | {:<24.2}",
        avg_edge_value_fitness, avg_connectivity_fitness, avg_overall_deviation_fitness
    );    


}


pub fn run_genetic_algorithm_instance(config: &Config) {
    println!("Starting Genetic Algorithm Instance"); 
    print!("Initializing Population...");
    let mut population: Population = initialize_random_population(&config);
    print!("DONE\nInitial Population Statistics: ");


    for generation in 0..config.number_of_generations {
        let current_population_ranked = non_dominated_sort(&population);
        log_population_statistics(&population, &current_population_ranked);
        println!("Calculating Generation: {:?}", generation);

        print!("SEL|");
        io::stdout().flush().unwrap();
        let mut parents = parent_selection(&population, &current_population_ranked, &config);

        print!("CROSS|");
        io::stdout().flush().unwrap();
        let mut children = crossover(&mut parents, &config);

        print!("MUT|");
        io::stdout().flush().unwrap();
        children = mutate(&mut children, &config);

        println!("SURV_SEL");
        io::stdout().flush().unwrap();
        population = survivor_selection(&population, &children, &config);

        
    }
}


