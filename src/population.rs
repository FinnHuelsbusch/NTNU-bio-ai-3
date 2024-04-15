use crate::{
    config::Config,
    distance::calculate_euclidean_distance_map_for_neighbors,
    individual::Individual,
};

pub type Population = Vec<Individual>;

pub fn initialize_random_population(config: &Config) -> Population {
    // calculate euclidian distance map for the image and copy it to each individual
    let rgb_image = Individual::open_image_as_rgb(&config.picture_path);
    let distance_map = calculate_euclidean_distance_map_for_neighbors(rgb_image);

    let mut population = Vec::new();
    for _ in 0..config.population_size {
        let mut individual = Individual::new(&config.picture_path);
        individual.euclidean_distance = distance_map.clone();
        individual.update_objectives();
        population.push(individual);
    }
    population
}

pub fn non_dominated_sort(population: &Population) -> Vec<Vec<Individual>> {
    let mut working_population = population.clone();
    let mut fronts: Vec<Vec<Individual>> = vec![];
    while working_population.is_empty() == false{
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
