use rand::Rng;

use crate::{
    config::Config,
    global_data::GlobalData,
    individual::{ Connection, Genome },
    population::Population,
};

fn flip_one_bit(genome: &mut Genome) {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..genome.len());
    let new_connection = match rand::thread_rng().gen_range(0..5) {
        0 => Connection::None,
        1 => Connection::Up,
        2 => Connection::Down,
        3 => Connection::Left,
        4 => Connection::Right,
        _ => panic!("Invalid connection value"),
    };
    genome[index] = new_connection;
}

pub fn mutate(population: &mut Population, config: &Config, global_data: &GlobalData) {
    let mut rng = rand::thread_rng();
    for mutation_config in config.mutations.iter() {
        // Calculate the number of crossovers which should happen for the specific config
        let number_of_mutations: u64 = (
            (config.population_size as f64) * mutation_config.probability.unwrap()
        ).ceil() as u64;

        println!("Number of mutations: {:?}", number_of_mutations);
        println!("Mutation probability: {:?}", mutation_config.probability);
        for _ in 0..number_of_mutations {
            let individual_index: usize = rng.gen_range(0..config.population_size);
            let child_genome = &mut population[individual_index].genome;
            match mutation_config.name.as_str() {
                "flip_one_bit" => {
                    flip_one_bit(child_genome);
                }
                _ =>
                    panic!(
                        "Didn't have an Implementation for mutation function: {:?}",
                        mutation_config.name.as_str()
                    ),
            }

            population[individual_index].update_objectives(global_data);
        }
    }
}
