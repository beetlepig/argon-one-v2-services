use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::fs::File;
use std::io::Read;

const YAML_CONFIG_PATH: &str = "/etc/argon_services_config.yaml";

pub type TempMatrix = Vec<[u8; 2]>;

#[derive(Debug, Deserialize)]
pub struct Hysteresis {
    pub amount: u8,
    pub only_way_down: bool,
}
#[derive(Debug, Deserialize)]
pub struct FanConfig {
    pub interval: u32,
    pub hysteresis: Hysteresis,
    #[serde(deserialize_with = "deserialize_matrix")]
    pub matrix: TempMatrix,
}
#[derive(Debug, Deserialize)]
pub struct PowerScript {
    pub location: String,
    pub args: Vec<String>,
}
#[derive(Debug, Deserialize)]
pub struct ArgonConfig {
    pub fan_config: FanConfig,
    pub shutdown_script: Option<PowerScript>,
    pub reboot_script: Option<PowerScript>,
}
fn deserialize_matrix<'de, D>(deserializer: D) -> Result<TempMatrix, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let data: TempMatrix = Vec::deserialize(deserializer)?;

    if data.len() < 2 {
        return Err(Error::custom(format!(
            "You must specify at least two points in the temperature matrix. Found: {}",
            data.len()
        )));
    }

    for row in &data {
        for &value in row {
            if value > 100 {
                return Err(Error::custom(format!(
                    "Only numbers within 0-100. Found: {}",
                    value
                )));
            }
        }
    }

    Ok(data)
}

pub fn load_argon_config() -> ArgonConfig {
    let default_config = ArgonConfig {
        fan_config: FanConfig {
            interval: 10000u32,
            hysteresis: Hysteresis {
                amount: 4,
                only_way_down: true,
            },
            matrix: vec![[55u8, 10u8], [60u8, 40u8], [65u8, 100u8]],
        },
        shutdown_script: None,
        reboot_script: None,
    };

    let argon_config_ok: ArgonConfig = match read_validate_yaml(YAML_CONFIG_PATH) {
        Ok(argon_config) => ArgonConfig {
            fan_config: FanConfig {
                matrix: sort_matrix(&argon_config.fan_config.matrix),
                ..argon_config.fan_config
            },
            ..argon_config
        },
        Err(e) => {
            eprintln!("Not a valid YAML: {}", e);
            println!("Fallback to default config...");
            default_config
        }
    };

    argon_config_ok
}

fn read_validate_yaml(config_path: &str) -> Result<ArgonConfig, Box<dyn std::error::Error>> {
    let mut file = File::open(config_path)?;

    let mut yaml_content = String::new();
    file.read_to_string(&mut yaml_content)?;

    let fan_config: ArgonConfig = serde_yaml::from_str(&yaml_content)?;

    Ok(fan_config)
}

fn sort_matrix(matrix: &TempMatrix) -> TempMatrix {
    let mut sorted_matrix = matrix.to_owned();

    sorted_matrix.sort_by(|a, b| {
        let cmp = a[0].cmp(&b[0]);

        if cmp == Ordering::Equal {
            a[1].cmp(&b[1])
        } else {
            cmp
        }
    });

    sorted_matrix
}
