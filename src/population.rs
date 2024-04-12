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

pub fn non_dominated_sort(population: &Population) -> Vec<Vec<usize>> {
    let mut domination_count: Vec<u64> = vec![0; population.len()];
    let mut dominated_by: Vec<Vec<usize>> = vec![Vec::new(); population.len()];

    let mut fronts: Vec<Vec<usize>> = vec![vec![]];

    let mut front_id: usize = 0;
    for i in 0..population.len() {
        for j in 0..population.len() {
            if i == j {
                continue;
            }

            if population[i].dominates(&population[j]) {
                domination_count[i] += 1;
                dominated_by[j].push(i);
            } else if population[j].dominates(&population[i]) {
                break;
            }
        }

        if domination_count[i] == 0 {
            fronts[front_id].push(i);
        } else {
            let mut new_front = true;
            for front in fronts.iter_mut() {
                // check if i has dominated anything in the current front
                let mut i_dominates_any_in_front = false;
                for individual_in_front in front.iter() {
                    if population[i].dominates(&population[*individual_in_front]) {
                        i_dominates_any_in_front = true;
                        break;
                    }
                }

                if !i_dominates_any_in_front {
                    front.push(i);
                    new_front = false;
                    break;
                }
            }

            if new_front {
                front_id += 1;
                fronts.push(vec![i]);
            }
        }
    }

    fronts
}
