use std::io::{self, Write};

use crate::crossover_functions::crossover;
use crate::mutation_functions::mutate;
use crate::selection_functions::{parent_selection, survivor_selection};
use crate::{config::Config, population::Population};
use crate::population::{initialize_random_population, non_dominated_sort};



pub fn run_genetic_algorithm_instance(config: &Config) {
    let mut population: Population = initialize_random_population(&config);


    for generation in 0..config.number_of_generations {
        let current_population_ranked = non_dominated_sort(&population);
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


