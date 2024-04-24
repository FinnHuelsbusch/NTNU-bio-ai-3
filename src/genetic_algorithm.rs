use std::io::{ self, Write };

use crate::crossover_functions::crossover;

use crate::global_data::GlobalData;
use crate::individual::Individual;
use crate::utils::show;

use crate::mutation_functions::{ eat_similar, mutate };
use crate::selection_functions::{ parent_selection, survivor_selection };
use crate::{ config::Config, population::Population };
use crate::population::{
    self,
    initialize_population,
    non_dominated_sort,
    save_individuals_to_files,
};

fn log_population_statistics(
    population: &Population,
    current_population_ranked: &Vec<Vec<Individual>>,
    iteration: usize
) {
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
    let mut min_weighted_fitness = f64::MAX;
    let mut max_weighted_fitness = f64::MIN;
    let mut avg_weighted_fitness = 0.0;
    let mut file_output = String::new();

    for rank in 0..current_population_ranked.len() {
        for individual in current_population_ranked[rank].iter() {
            let fitness = individual.get_objectives();
            let edge_value_fitness = fitness.0;
            let connectivity_fitness = fitness.1;
            let overall_deviation_fitness = fitness.2;
            let weighted_fitness = individual.fitness;
            file_output += &format!(
                "({},{},{});",
                edge_value_fitness,
                connectivity_fitness,
                overall_deviation_fitness
            );

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

            if weighted_fitness < min_weighted_fitness {
                min_weighted_fitness = weighted_fitness;
            }
            if weighted_fitness > max_weighted_fitness {
                max_weighted_fitness = weighted_fitness;
            }
            avg_weighted_fitness += weighted_fitness;
        }
        file_output += "\n";
    }

    let mut file = std::fs::File::create(format!("./logs/pareto_front_{}.txt", iteration)).unwrap();
    file.write_all(file_output.as_bytes()).unwrap();

    avg_connectivity_fitness /= population.len() as f64;
    avg_overall_deviation_fitness /= population.len() as f64;
    avg_edge_value_fitness /= population.len() as f64;
    avg_weighted_fitness /= population.len() as f64;

    // print as table
    println!(
        "Statistics: | Edge Value Fitness | Connectivity Fitness | Overall Deviation Fitness | Weighted Fitness"
    );
    println!(
        "Best:       | {:<18.2} | {:<20.2} | {:<25.2} | {:<24.2}",
        max_edge_value_fitness,
        min_connectivity_fitness,
        min_overall_deviation_fitness,
        max_weighted_fitness
    );
    println!(
        "Avg:        | {:<18.2} | {:<20.2} | {:<25.2} | {:<24.2}",
        avg_edge_value_fitness,
        avg_connectivity_fitness,
        avg_overall_deviation_fitness,
        avg_weighted_fitness
    );
    println!(
        "Worst:      | {:<18.2} | {:<20.2} | {:<25.2} | {:<24.2}",
        min_edge_value_fitness,
        max_connectivity_fitness,
        max_overall_deviation_fitness,
        min_weighted_fitness
    );
}

pub fn run_genetic_algorithm_instance(config: &Config, global_data: &GlobalData) {
    println!("Starting Genetic Algorithm Instance");
    print!("Initializing Population...");
    let mut population: Population = initialize_population(config, global_data);

    print!("DONE\nInitial Population Statistics: ");

    for generation in 0..config.number_of_generations {
        let current_population_ranked = non_dominated_sort(&population);
        println!("{:?}, {:?}", population.len(), current_population_ranked.len());
        log_population_statistics(&population, &current_population_ranked, generation);

        println!("Calculating Generation: {:?}", generation);

        print!("SEL|");
        io::stdout().flush().unwrap();
        let parents = parent_selection(&population, &current_population_ranked, config);

        print!("CROSS|");
        io::stdout().flush().unwrap();
        let mut children = parents.clone();
        crossover(&mut children, config, global_data);

        print!("MUT|");
        io::stdout().flush().unwrap();
        mutate(&mut children, config, global_data);

        print!("EVAL|");
        io::stdout().flush().unwrap();
        for individual in children.iter_mut() {
            if individual.needs_update() {
                individual.update_objectives(config, global_data);
            }
        }

        println!("SURV_SEL");
        io::stdout().flush().unwrap();
        population = survivor_selection(&population, &children, config);
    }

    let pareto_fronts = non_dominated_sort(&population);
    let _ = save_individuals_to_files(&pareto_fronts[0], config, global_data);
    for individual in pareto_fronts[0].iter() {
        show(&individual.get_segment_border_image_inline(global_data));
    }
}
