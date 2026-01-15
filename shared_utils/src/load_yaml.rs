use crate::mapper::matrix_mapper;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

const YAML_CONFIG_PATH: &str = "/etc/argonone/argon_services_config.yaml";

pub type TemperatureMatrixVec = Vec<[u8; 2]>;
pub type TemperatureMatrixHashMap = HashMap<u8, u8>;

#[derive(Debug, Deserialize)]
pub struct Hysteresis {
    pub amount: u8,
    pub only_way_down: bool,
}
#[derive(Debug, Deserialize)]
pub struct FanConfig {
    pub interval: u64,
    pub hysteresis: Hysteresis,
    #[serde(deserialize_with = "deserialize_matrix")]
    pub matrix: TemperatureMatrixHashMap,
}
fn deserialize_matrix<'de, D>(deserializer: D) -> Result<TemperatureMatrixHashMap, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MatrixHelper {
        Vec(TemperatureMatrixVec),
        Map(TemperatureMatrixHashMap),
    }

    let deserialized_data: MatrixHelper = MatrixHelper::deserialize(deserializer)?;

    match deserialized_data {
        MatrixHelper::Vec(temperature_vec) => {
            if temperature_vec.len() < 2 {
                return Err(Error::custom(format!(
                    "You must specify at least two points in the temperature matrix. Found: {}",
                    temperature_vec.len()
                )));
            }

            for row in &temperature_vec {
                for &value in row {
                    if value > 100 {
                        return Err(Error::custom(format!(
                            "Only numbers within 0-100. Found: {}",
                            value
                        )));
                    }
                }
            }

            let temperature_hashmap = matrix_mapper(temperature_vec);

            Ok(temperature_hashmap)
        }
        MatrixHelper::Map(temperature_hashmap) => Ok(temperature_hashmap),
    }
}
impl Default for FanConfig {
    fn default() -> Self {
        FanConfig {
            interval: 10000u64,
            hysteresis: Hysteresis {
                amount: 4u8,
                only_way_down: true,
            },
            matrix: matrix_mapper(vec![[55u8, 10u8], [60u8, 40u8], [65u8, 100u8]]),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PowerScript {
    pub location: String,
    pub args: Vec<String>,
}
pub type PowerScriptOption = Option<PowerScript>;
#[derive(Debug, Deserialize)]
pub struct ArgonConfig {
    pub fan_config: FanConfig,
    pub shutdown_script: PowerScriptOption,
    pub reboot_script: PowerScriptOption,
}
impl Default for ArgonConfig {
    fn default() -> Self {
        ArgonConfig {
            fan_config: FanConfig::default(),
            shutdown_script: None,
            reboot_script: None,
        }
    }
}

pub fn load_argon_config() -> ArgonConfig {
    read_validate_yaml(YAML_CONFIG_PATH).unwrap_or_else(|e| {
        eprintln!("Not a valid YAML: {}", e);
        println!("Fallback to default config...");
        ArgonConfig::default()
    })
}

fn read_validate_yaml(config_path: &str) -> Result<ArgonConfig, Box<dyn std::error::Error>> {
    let mut file = File::open(config_path)?;

    let mut yaml_content = String::new();
    file.read_to_string(&mut yaml_content)?;

    let fan_config: ArgonConfig = serde_yaml::from_str(&yaml_content)?;

    Ok(fan_config)
}
