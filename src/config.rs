use serde::{ Deserialize, Serialize };
use serde_json::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionConfig {
    pub name: String,

    #[serde(default)]
    pub probability: Option<f64>,

    #[serde(default)]
    pub tournament_size: Option<usize>,

    #[serde(default)]
    pub combine_parents_and_offspring: Option<bool>,

    #[serde(default)]
    pub number_of_slices: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub problem_instance: String,
    pub population_size: usize,
    pub number_of_generations: usize,
    pub initialization_method: String,
    pub parent_selection: FunctionConfig,
    pub crossovers: Vec<FunctionConfig>,
    pub mutations: Vec<FunctionConfig>,
    pub survivor_selection: FunctionConfig,
    pub preserve_skyline: bool,
    pub edge_value_multiplier: f64,
    pub connectivity_multiplier: f64,
    pub overall_deviation_multiplier: f64,
}

pub fn initialize_config(file_path: &str) -> Config {
    let data = std::fs::read_to_string(file_path).expect("Unable to read file");
    let new_instance: Result<Config, Error> = serde_json::from_str(&data);
    match new_instance {
        Ok(instance) => instance,
        Err(e) => {
            panic!("Error parsing config file: {:?}", e);
        }
    }
}
