use crate::{ config::Config, individual::Genome, population::Population };
use rand::Rng;

pub fn one_point_crossover(genome1: &Genome, genome2: &Genome) -> (Genome, Option<Genome>) {
    assert_eq!(genome1.len(), genome2.len());
    let mut rng = rand::thread_rng();
    let slice_index = rng.gen_range(0..genome1.len());
    let mut child1 = genome1.clone();
    let mut child2 = genome2.clone();
    for i in slice_index..genome1.len() {
        child1[i] = genome2[i];
        child2[i] = genome1[i];
    }

    (child1, Some(child2))
}

pub fn n_point_crossover(
    genome1: &Genome,
    genome2: &Genome,
    number_of_slices: usize
) -> (Genome, Option<Genome>) {
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
    (child1, Some(child2))
}

pub fn uniform_crossover(genome1: &Genome, genome2: &Genome) -> (Genome, Option<Genome>) {
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
    (child1, Some(child2))
}

pub fn crossover(population: &mut Population, config: &Config) -> Population {
    let mut rng = rand::thread_rng();
    let mut children: Population = population.clone();
    for crossover_config in config.crossovers.iter() {
        // Calculate the number of crossovers which should happen for the specific config
        let number_of_crossovers: u64 = (
            (config.population_size as f64) * crossover_config.probability.unwrap_or(0.0)
        ).ceil() as u64;

        for _ in 0..number_of_crossovers {
            let individual_index_a: usize = rng.gen_range(0..config.population_size);
            let mut individual_index_b: usize = rng.gen_range(0..config.population_size);

            while individual_index_a == individual_index_b {
                individual_index_b = rng.gen_range(0..config.population_size);
            }

            let child_genomes: (Genome, Option<Genome>) = match crossover_config.name.as_str() {
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
                        "Didn't have an Implementation for selection function: {:?}",
                        config.parent_selection.name.as_str()
                    ),
            };

            let mut child_a = population[individual_index_a].clone();
            child_a.genome = child_genomes.0;
            child_a.update_objectives();
            children.push(child_a);

            if let Some(genome) = child_genomes.1 {
                let mut child_b = population[individual_index_b].clone();
                child_b.genome = genome;
                child_b.update_objectives();
                children.push(child_b);
            }
        }
    }

    children
}
