use crate::mapper::matrix_mapper;
use rkyv::validation::validators::DefaultValidator;
use rkyv::validation::CheckTypeError;
use rkyv::{AlignedVec, Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize as SerdeDeserialize, Deserializer as SerdeDeserializer};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::{metadata, read, remove_file, write};
use std::io;

const YAML_CONFIG_PATH: &str = "/etc/argonone/argon_services_config.yaml";
pub const RKYV_CONFIG_PATH: &str = "/etc/argonone/argon_services_config.rkyv";

pub type TempMatrixYAML = Vec<[u8; 2]>;
pub type TempMatrixRKYV = HashMap<u8, u8>;

#[derive(SerdeDeserialize, RkyvDeserialize, RkyvSerialize, Archive, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Hysteresis {
    pub amount: u8,
    pub only_way_down: bool,
}
#[derive(SerdeDeserialize, Debug)]
pub struct FanConfigYAML {
    pub interval: u64,
    pub hysteresis: Hysteresis,
    #[serde(deserialize_with = "deserialize_matrix")]
    pub matrix: TempMatrixYAML,
}
fn deserialize_matrix<'de, D>(deserializer: D) -> Result<TempMatrixYAML, D::Error>
where
    D: SerdeDeserializer<'de>,
{
    let data: TempMatrixYAML = SerdeDeserialize::deserialize(deserializer)?;

    if data.len() < 2 {
        return Err(serde::de::Error::custom(format!(
            "You must specify at least two points in the temperature matrix. Found: {}",
            data.len()
        )));
    }

    for row in &data {
        for &value in row {
            if value > 100 {
                return Err(serde::de::Error::custom(format!(
                    "Only numbers within 0-100. Found: {}",
                    value
                )));
            }
        }
    }

    Ok(data)
}

#[derive(RkyvDeserialize, RkyvSerialize, Archive, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct FanConfigRKYV {
    pub interval: u64,
    pub hysteresis: Hysteresis,
    pub matrix: TempMatrixRKYV,
}
impl Default for FanConfigRKYV {
    fn default() -> Self {
        FanConfigRKYV {
            interval: 10000u64,
            hysteresis: Hysteresis {
                amount: 4u8,
                only_way_down: true,
            },
            matrix: matrix_mapper(vec![[55u8, 10u8], [60u8, 40u8], [65u8, 100u8]]),
        }
    }
}

#[derive(SerdeDeserialize, RkyvDeserialize, RkyvSerialize, Archive, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct PowerScript {
    pub location: String,
    pub args: Vec<String>,
}
#[derive(SerdeDeserialize, Debug)]
pub struct ArgonConfigYAML {
    pub fan_config: FanConfigYAML,
    pub shutdown_script: Option<PowerScript>,
    pub reboot_script: Option<PowerScript>,
}

#[derive(RkyvDeserialize, RkyvSerialize, Archive, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct ArgonConfigRKYV {
    pub fan_config: FanConfigRKYV,
    pub shutdown_script: Option<PowerScript>,
    pub reboot_script: Option<PowerScript>,
}
impl Default for ArgonConfigRKYV {
    fn default() -> Self {
        ArgonConfigRKYV {
            fan_config: FanConfigRKYV::default(),
            shutdown_script: None,
            reboot_script: None,
        }
    }
}

pub fn load_argon_config<F: FnOnce(ArgonConfigValue)>(on_config_ready: F) {
    let initial_config = load_initial_config();
    match_argon_config(initial_config, on_config_ready, RKYV_CONFIG_PATH);
}

pub fn load_initial_config() -> ConfigTypes {
    deserialize_argon_config(
        RKYV_CONFIG_PATH,
        read_file(RKYV_CONFIG_PATH, YAML_CONFIG_PATH),
    )
    .unwrap_or_else(|| {
        println!("Fallback to default config...");
        ConfigTypes::Serialized(ArgonConfigRKYV::default())
    })
}

fn remove_cache_on_error<F: FnOnce(ArgonConfigValue)>(
    archived_argon_config_result: Result<
        ArgonConfigValue,
        CheckTypeError<ArchivedArgonConfigRKYV, DefaultValidator>,
    >,
    on_config_ready: F,
    rkyv_config_path: &str,
) -> () {
    match archived_argon_config_result {
        Ok(argon_config_value) => {
            on_config_ready(argon_config_value);
        }
        Err(e) => {
            println!("Archive creation failed: {}", e);
            match remove_file(rkyv_config_path) {
                Ok(_) => {
                    let initial_argon_config = load_initial_config();
                    match_argon_config(initial_argon_config, on_config_ready, rkyv_config_path);
                }
                Err(e) => {
                    eprintln!(
                        "Error removing cache file, fallback to default config: {}",
                        e
                    );
                    on_config_ready(ArgonConfigValue::NonArchived(ArgonConfigRKYV::default()));
                }
            }
        }
    }
}

#[derive(RkyvDeserialize, RkyvSerialize, Archive, Debug)]
#[archive(check_bytes)]
pub enum ArgonConfigValue<'a> {
    Archived(&'a ArchivedArgonConfigRKYV),
    NonArchived(ArgonConfigRKYV),
}
fn match_argon_config<F: FnOnce(ArgonConfigValue)>(
    loaded_argon_config: ConfigTypes,
    on_config_ready: F,
    rkyv_config_path: &str,
) {
    match loaded_argon_config {
        ConfigTypes::Serialized(argon_config) => {
            remove_cache_on_error(
                Ok(ArgonConfigValue::NonArchived(argon_config)),
                on_config_ready,
                rkyv_config_path,
            );
        }
        ConfigTypes::NonSerialized(RkyvBuffers::Aligned(rkyv_aligned_buffer)) => {
            let archived_argon_config_result =
                rkyv::check_archived_root::<ArgonConfigRKYV>(&rkyv_aligned_buffer)
                    .map(|archived_argon_config| ArgonConfigValue::Archived(archived_argon_config));
            remove_cache_on_error(
                archived_argon_config_result,
                on_config_ready,
                rkyv_config_path,
            );
        }
        ConfigTypes::NonSerialized(RkyvBuffers::Raw(rkyv_raw_buffer)) => {
            let archived_argon_config_result =
                rkyv::check_archived_root::<ArgonConfigRKYV>(&rkyv_raw_buffer)
                    .map(|archived_argon_config| ArgonConfigValue::Archived(archived_argon_config));
            remove_cache_on_error(
                archived_argon_config_result,
                on_config_ready,
                rkyv_config_path,
            );
        }
    }
}

pub enum RkyvBuffers {
    Raw(Vec<u8>),
    Aligned(AlignedVec),
}
pub enum ConfigTypes {
    Serialized(ArgonConfigRKYV),
    NonSerialized(RkyvBuffers),
}
fn deserialize_argon_config(
    rkyv_path: &str,
    config_file_buffer_result: Result<Vec<u8>, ReadFileError>,
) -> Option<ConfigTypes> {
    match config_file_buffer_result {
        Ok(rkyv_buffer) => {
            println!("Cache file found");
            Some(ConfigTypes::NonSerialized(RkyvBuffers::Raw(rkyv_buffer)))
        }
        Err(ReadFileError::NoCacheFoundError(yaml_buffer)) => {
            let argon_config_result = serde_yaml::from_slice::<ArgonConfigYAML>(&yaml_buffer);
            match argon_config_result {
                Ok(argon_config) => {
                    println!("Not valid cache file found, creating file...");

                    let rkyv_config = ArgonConfigRKYV {
                        shutdown_script: argon_config.shutdown_script,
                        reboot_script: argon_config.reboot_script,
                        fan_config: FanConfigRKYV {
                            interval: argon_config.fan_config.interval,
                            hysteresis: argon_config.fan_config.hysteresis,
                            matrix: matrix_mapper(argon_config.fan_config.matrix),
                        },
                    };
                    let combined_result = rkyv::to_bytes::<ArgonConfigRKYV, 5120>(&rkyv_config)
                        .map_err(|e| e.to_string())
                        .and_then(|archived_bytes| {
                            write(rkyv_path, &archived_bytes)
                                .map_err(|e| e.to_string())
                                .map(|_| archived_bytes)
                        });

                    match combined_result {
                        Ok(archived_bytes) => Some(ConfigTypes::NonSerialized(
                            RkyvBuffers::Aligned(archived_bytes),
                        )),
                        Err(e) => {
                            eprintln!("Error saving argon config cache: {}", e);
                            Some(ConfigTypes::Serialized(rkyv_config))
                        }
                    }
                }
                Err(e) => {
                    eprintln!("There is an error with your YAML config: {}", e);
                    None
                }
            }
        }
        Err(ReadFileError::YamlIoError(yaml_error)) => {
            println!("Error reading YAML config file: {}", yaml_error);
            None
        }
    }
}

enum ReadFileError {
    YamlIoError(io::Error),
    NoCacheFoundError(Vec<u8>),
}
fn read_file(rkyv_path: &str, yaml_path: &str) -> Result<Vec<u8>, ReadFileError> {
    let yaml_modified = metadata(yaml_path)
        .and_then(|meta| meta.modified())
        .map_err(|e| ReadFileError::YamlIoError(e))?;
    let rkyv_modified_result = metadata(rkyv_path).and_then(|meta| meta.modified());

    if let Ok(rkyv_modified) = rkyv_modified_result {
        if let Ordering::Greater | Ordering::Equal = rkyv_modified.cmp(&yaml_modified) {
            if let Ok(rkyv_file) = read(rkyv_path) {
                return Ok(rkyv_file);
            }
        }
    }

    let yaml_buffer = read(yaml_path).map_err(|e| ReadFileError::YamlIoError(e))?;
    Err(ReadFileError::NoCacheFoundError(yaml_buffer))
}
