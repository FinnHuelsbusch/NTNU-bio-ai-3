use std::io::{ self, Write };

use crate::crossover_functions::crossover;

use crate::global_data::GlobalData;
use crate::individual::Individual;
use crate::utils::show;
use crate::individual;
use crate::mutation_functions::mutate;
use crate::selection_functions::{ parent_selection, survivor_selection };
use crate::{ config::Config, population::Population };
use crate::population::{ initialize_random_population, non_dominated_sort };

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
    let mut file_output = String::new();

    for rank in 0..current_population_ranked.len() {
        for individual in current_population_ranked[rank].iter() {
            file_output += &format!(
                "({},{},{});",
                individual.edge_value_fitness,
                individual.connectivity_fitness,
                individual.overall_deviation_fitness
            );
            let edge_value_fitness = individual.edge_value_fitness;
            let connectivity_fitness = individual.connectivity_fitness;
            let overall_deviation_fitness = individual.overall_deviation_fitness;

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
        file_output += "\n";
    }

    let mut file = std::fs::File::create(format!("./logs/pareto_front_{}.txt", iteration)).unwrap();
    file.write_all(file_output.as_bytes()).unwrap();

    avg_connectivity_fitness /= population.len() as f64;
    avg_overall_deviation_fitness /= population.len() as f64;
    avg_edge_value_fitness /= population.len() as f64;

    // print as table
    println!("Statistics: | Edge Value Fitness | Connectivity Fitness | Overall Deviation Fitness");
    println!(
        "Min:        | {:<18.2} | {:<19.2} | {:<24.2}",
        min_edge_value_fitness,
        min_connectivity_fitness,
        min_overall_deviation_fitness
    );
    println!(
        "Max:        | {:<18.2} | {:<19.2} | {:<24.2}",
        max_edge_value_fitness,
        max_connectivity_fitness,
        max_overall_deviation_fitness
    );
    println!(
        "Avg:        | {:<18.2} | {:<19.2} | {:<24.2}",
        avg_edge_value_fitness,
        avg_connectivity_fitness,
        avg_overall_deviation_fitness
    );
}

pub fn run_genetic_algorithm_instance(config: &Config, global_data: &GlobalData) {
    println!("Starting Genetic Algorithm Instance");
    print!("Initializing Population...");
    let mut population: Population = initialize_random_population(config, global_data);

    print!("DONE\nInitial Population Statistics: ");

    for generation in 0..config.number_of_generations {
        let current_population_ranked = non_dominated_sort(&population);
        println!("{:?}, {:?}", population.len(), current_population_ranked.len());
        log_population_statistics(&population, &current_population_ranked, generation);

        println!("Calculating Generation: {:?}", generation);

        print!("SEL|");
        io::stdout().flush().unwrap();
        let mut parents = parent_selection(&population, &current_population_ranked, config);

        print!("CROSS|");
        io::stdout().flush().unwrap();
        let mut children = crossover(&mut parents, config, global_data);

        print!("MUT|");
        io::stdout().flush().unwrap();
        mutate(&mut children, config, global_data);

        println!("SURV_SEL");
        io::stdout().flush().unwrap();
        population = survivor_selection(&population, &children, config);
        print!("Number of None in genes of children: ");
        // print number of None in gene of each individual
        for individual in children.iter() {
            let mut none_count = 0;
            for gene in individual.genome.iter() {
                if gene == &individual::Connection::None {
                    none_count += 1;
                }
            }
            print!("{:?},", none_count);
        }
        println!();
    }
    let pareto_fronts = non_dominated_sort(&population);
    for individual in pareto_fronts[0].iter() {
        show(&individual.get_segments_image(global_data));
    }
}
