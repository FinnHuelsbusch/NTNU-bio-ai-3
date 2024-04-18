use crate::{ config::Config, global_data::GlobalData, individual::Genome, population::Population };
use rand::Rng;

pub fn one_point_crossover(genome1: &Genome, genome2: &Genome) -> (Genome, Genome) {
    assert_eq!(genome1.len(), genome2.len());

    let mut rng = rand::thread_rng();
    let rng_num: usize = rng.gen_range(0..genome1.len());
    let mut new_genome1 = genome1.clone();
    let mut new_genome2 = genome2.clone();


    new_genome1[rng_num..].swap_with_slice(&mut new_genome2[rng_num..]);

    (new_genome1.to_vec(), new_genome2.to_vec())
}

pub fn n_point_crossover(
    genome1: &Genome,
    genome2: &Genome,
    number_of_slices: usize
) -> (Genome, Genome) {
    assert_eq!(genome1.len(), genome2.len());
    let mut rng = rand::thread_rng();
    let mut slices = Vec::new();
    for _ in 0..number_of_slices {
        slices.push(rng.gen_range(0..genome1.len()));
    }
    slices.sort();
    let mut child1 = genome1.clone();
    let mut child2 = genome2.clone();
    let mut current_parent = 0;
    for i in 0..genome1.len() {
        if current_parent == 0 {
            child1[i] = genome1[i];
            child2[i] = genome2[i];
        } else {
            child1[i] = genome2[i];
            child2[i] = genome1[i];
        }
        if slices.contains(&i) {
            current_parent = 1 - current_parent;
        }
    }
    (child1, child2)
}

pub fn uniform_crossover(genome1: &Genome, genome2: &Genome) -> (Genome, Genome) {
    assert_eq!(genome1.len(), genome2.len());
    let mut rng = rand::thread_rng();
    let mut child1 = genome1.clone();
    let mut child2 = genome2.clone();
    for i in 0..genome1.len() {
        if rng.gen::<f64>() < 0.5 {
            child1[i] = genome2[i];
            child2[i] = genome1[i];
        }
    }
    (child1, child2)
}

pub fn crossover(
    population: &mut Population,
    config: &Config,
    global_data: &GlobalData
) {
    let mut rng = rand::thread_rng();
    for crossover_config in config.crossovers.iter() {
        // Calculate the number of crossovers which should happen for the specific config
        let number_of_crossovers: u64 = (
            ((config.population_size as f64) * crossover_config.probability.unwrap()) /
            2.0
        ).ceil() as u64;

        for _ in 0..number_of_crossovers {
            let individual_index_a: usize = rng.gen_range(0..config.population_size);
            let mut individual_index_b: usize = rng.gen_range(0..config.population_size);

            while individual_index_a == individual_index_b {
                individual_index_b = rng.gen_range(0..config.population_size);
            }

            let child_genomes: (Genome, Genome) = match crossover_config.name.as_str() {
                "one_point" =>
                    one_point_crossover(
                        &population[individual_index_a].genome,
                        &population[individual_index_b].genome
                    ),
                "n_point" =>
                    n_point_crossover(
                        &population[individual_index_a].genome,
                        &population[individual_index_b].genome,
                        crossover_config.number_of_slices.unwrap()
                    ),
                "uniform" =>
                    uniform_crossover(
                        &population[individual_index_a].genome,
                        &population[individual_index_b].genome
                    ),

                // Handle the rest of cases
                _ =>
                    panic!(
                        "Didn't have an Implementation for crossover function: {:?}",
                        config.parent_selection.name.as_str()
                    ),
            };

            population[individual_index_a].genome = child_genomes.0;
            population[individual_index_a].needs_update();
            population[individual_index_b].genome = child_genomes.1;
            population[individual_index_b].needs_update();
            
        }
    }
}
