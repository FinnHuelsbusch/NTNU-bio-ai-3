use rand::Rng;

use crate::{
    config::Config,
    individual::Individual,
    population::{ non_dominated_sort, Population },
};

fn tournament_selection(
    population: &Population,
    population_size: usize,
    tournament_size: usize,
    tournament_probability: f64
) -> Population {
    let mut new_population: Population = Vec::with_capacity(population_size);
    let mut rng = rand::thread_rng();

    for _ in 0..population_size {
        let mut tournament: Vec<Individual> = Vec::with_capacity(tournament_size);
        for _ in 0..tournament_size {
            let index = rng.gen_range(0..population.len());
            tournament.push(population[index].clone());
        }

        let sorted_tournament = non_dominated_sort(&tournament);
        let selected_individual = if rng.gen::<f64>() < tournament_probability {
            &sorted_tournament[0][rng.gen_range(0..sorted_tournament[0].len())]
        } else {
            // if the number of frontiers is greater than 1, select a random rank
            if sorted_tournament.len() > 1 {
                let rank = rng.gen_range(1..sorted_tournament.len());
                &sorted_tournament[rank][rng.gen_range(0..sorted_tournament[rank].len())]
                // if the number of frontiers is 1, select a random individual from the first frontier
            } else {
                &sorted_tournament[0][rng.gen_range(0..sorted_tournament[0].len())]
            }
        };
        new_population.push(selected_individual.clone());
    }

    new_population
}

fn full_replacement_selection(
    population: &Population,
    children: &Population,
    population_size: usize
) -> Population {
    assert_eq!(population.len(), population_size);
    assert_eq!(children.len(), population_size);
    children.clone()
}

pub fn parent_selection(
    population: &Population,
    sorted_population: &Vec<Vec<Individual>>,
    config: &Config
) -> Population {
    let mut new_population: Population = Vec::with_capacity(config.population_size);
    if config.preserve_skyline {
        new_population.extend(sorted_population[0].clone());
    }
    let selected_population: Population = match config.parent_selection.name.as_str() {
        "tournament" =>
            tournament_selection(
                &population,
                config.population_size - new_population.len(),
                config.parent_selection.tournament_size.unwrap(),
                config.parent_selection.probability.unwrap()
            ),
        "none" => {
            if config.preserve_skyline {
                panic!("None selection is not compatible with preserving the skyline.");
            }
            population.clone()
        }
        // Handle the rest of cases
        _ =>
            panic!(
                "Didn't have an Implementation for selection function: {:?}",
                config.parent_selection.name.as_str()
            ),
    };
    new_population.extend(selected_population);
    new_population
}

pub fn survivor_selection(
    parents: &Population,
    children: &Population,
    config: &Config
) -> Population {
    let selection_population: Population;
    if config.survivor_selection.combine_parents_and_offspring.unwrap_or(false) {
        let mut combined_population: Population = parents.clone();
        combined_population.extend(children.clone());
        selection_population = combined_population;
    } else {
        selection_population = children.clone();
    }

    let mut new_population: Population = Vec::with_capacity(config.population_size);

    if config.preserve_skyline {
        let sorted_population = non_dominated_sort(&selection_population);
        new_population.extend(sorted_population[0].clone());
    }

    let selected_population: Population = match config.survivor_selection.name.as_str() {
        // Match a single value
        "fullReplacement" => {
            if config.preserve_skyline {
                panic!("Full Replacement selection is not compatible with preserving the skyline.");
            }
            full_replacement_selection(parents, children, config.population_size)
        }
        "tournament" => {
            tournament_selection(
                &selection_population,
                config.population_size - new_population.len(),
                config.survivor_selection.tournament_size.unwrap_or_else(||
                    panic!(
                        "You need to specify the tournament size if you are using tournament selection for survivor selection."
                    )
                ),
                config.survivor_selection.probability.unwrap_or_else(||
                    panic!(
                        "You need to specify the tournament probability if you are using tournament selection for survivor selection."
                    )
                )
            )
        }
        "NSGA-2" => {
            if config.preserve_skyline {
                panic!("NSGA-2 selection is not compatible with preserving the skyline.");
            }
            if !config.survivor_selection.combine_parents_and_offspring.unwrap_or(false) {
                panic!("NSGA-2 selection requires combining parents and offspring.");
            }
            let mut nsga2_population: Vec<Individual> = Vec::with_capacity(config.population_size);
            let sorted_population = non_dominated_sort(&selection_population);
            let mut i = 0;
            while nsga2_population.len() + sorted_population[i].len() <= config.population_size {
                nsga2_population.extend(sorted_population[i].clone());
                i += 1;
            }
            if nsga2_population.len() < config.population_size {
                let mut remaining_population: Vec<Individual> = Vec::new();
                for individual in sorted_population[i].iter() {
                    remaining_population.push(individual.clone());
                    if nsga2_population.len() + remaining_population.len() == config.population_size {
                        break;
                    }
                }
                nsga2_population.extend(remaining_population);
            }
            nsga2_population
        }
        _ =>
            panic!(
                "Didn't have an Implementation for selection function: {:?}",
                config.parent_selection.name.as_str()
            ),
    };
    new_population.extend(selected_population);
    new_population
}
