use crate::{ config::Config, global_data::GlobalData, individual::{self, get_mst_genome, Individual} };

pub type Population = Vec<Individual>;

pub fn initialize_population(config: &Config, global_data: &GlobalData) -> Population {
    // calculate euclidian distance map for the image and copy it to each individual

    let mut population = Vec::with_capacity(config.population_size);
    match config.initialization_method.as_str() {
        "random" => {
            for _ in 0..config.population_size {
                let mut individual = Individual::new_random(global_data);
                individual.update_objectives(global_data);
                population.push(individual);
            }
        }
        "mst" => {
            let genome = get_mst_genome(global_data.rgb_image, global_data.euclidean_distance_map);
            let mut individual = Individual::new_with_genome(&genome);
            individual.update_objectives(global_data);
            for _ in 0..config.population_size {
                population.push(individual.clone());
            }
        }
        _ => {
            panic!("Invalid initialization method");
        }
        
    }

    
    population
}

pub fn non_dominated_sort(population: &Population) -> Vec<Vec<Individual>> {
    let mut working_population = population.clone();
    let mut fronts: Vec<Vec<Individual>> = vec![];
    while working_population.is_empty() == false {
        let mut dominated_by: Vec<Vec<usize>> = vec![Vec::new(); working_population.len()];

        for i in 0..working_population.len() {
            for j in 0..working_population.len() {
                if i == j {
                    continue;
                }
                if working_population[i].dominates(&working_population[j]) {
                    dominated_by[j].push(i);
                }
            }
        }

        let mut current_front: Vec<Individual> = Vec::new();
        let mut new_working_population: Vec<Individual> = Vec::new();
        for i in 0..working_population.len() {
            if dominated_by[i].len() == 0 {
                current_front.push(working_population[i].clone());
            } else {
                new_working_population.push(working_population[i].clone());
            }
        }
        fronts.push(current_front);

        working_population = new_working_population;
    }
    fronts
}
