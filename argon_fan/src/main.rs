use shared_utils::initialize_device::initialize_i2c;
use shared_utils::load_yaml::{load_argon_config, ArgonConfigValue, TempMatrixRKYV};
use shared_utils::rkyv::collections::ArchivedHashMap;
use std::thread;
use std::time::Duration;
use systemstat::{Platform, System};

fn main() {
    load_argon_config(|final_argon_config| get_fan_attributes(final_argon_config));
}

fn control_fan(
    mapped_temperature_matrix: &SpeedTemperatureMatrix,
    hysteresis: u8,
    last_temperature: f32,
    only_way_down: bool,
) -> Result<f32, Box<dyn std::error::Error>> {
    let i2c = initialize_i2c()?;
    let sys = System::new();
    let device_temperature: f32 = sys.cpu_temp()?;
    let temperature_delta: f32 = device_temperature - last_temperature;

    println!(
        "Last target temperature: {}. Current temperature: {}. Delta: {}",
        last_temperature, device_temperature, temperature_delta
    );

    if only_way_down && temperature_delta > 0.0 || temperature_delta.abs() > hysteresis as f32 {
        let rounded_device_temp = device_temperature.round() as u8;
        let temp_value = match mapped_temperature_matrix {
            SpeedTemperatureMatrix::NonArchivedMatrix(matrix) => matrix.get(&rounded_device_temp),
            SpeedTemperatureMatrix::ArchivedMatrix(matrix) => matrix.get(&rounded_device_temp),
        };

        match temp_value {
            Some(&fan_speed) => {
                println!("Set new fan speed to device: {}", fan_speed);
                i2c.smbus_send_byte(fan_speed)?;
                return Ok(device_temperature);
            }
            None => {
                eprintln!("Temperature not found in matrix: {}", {
                    rounded_device_temp
                })
            }
        }
    }

    Ok(last_temperature)
}

enum SpeedTemperatureMatrix<'a> {
    NonArchivedMatrix(TempMatrixRKYV),
    ArchivedMatrix(&'a ArchivedHashMap<u8, u8>),
}
fn set_fan_speed_loop(
    interval: Duration,
    hysteresis: u8,
    only_way_down: bool,
    speed_temperature_matrix: SpeedTemperatureMatrix,
) {
    let mut retries: u8 = 0;
    let mut last_temperature: f32 = 0.0;

    while retries <= 3 {
        match control_fan(
            &speed_temperature_matrix,
            hysteresis,
            last_temperature,
            only_way_down,
        ) {
            Ok(new_temperature) => {
                last_temperature = new_temperature;
                retries = 0;
            }
            Err(e) => {
                eprintln!("Device error: {}", e);
                retries += retries + 1;
                thread::sleep(Duration::from_millis(10000));
            }
        };
        thread::sleep(interval);
    }
}
fn get_fan_attributes(argon_config_value: ArgonConfigValue) {
    match argon_config_value {
        ArgonConfigValue::Archived(archived_argon_config) => {
            let interval = Duration::from_millis(archived_argon_config.fan_config.interval);
            let hysteresis = archived_argon_config.fan_config.hysteresis.amount;
            let only_way_down = archived_argon_config.fan_config.hysteresis.only_way_down;
            let speed_temperature_matrix = &archived_argon_config.fan_config.matrix;
            set_fan_speed_loop(
                interval,
                hysteresis,
                only_way_down,
                SpeedTemperatureMatrix::ArchivedMatrix(speed_temperature_matrix),
            );
        }
        ArgonConfigValue::NonArchived(non_archived_argon_config) => {
            let interval = Duration::from_millis(non_archived_argon_config.fan_config.interval);
            let hysteresis = non_archived_argon_config.fan_config.hysteresis.amount;
            let only_way_down = non_archived_argon_config
                .fan_config
                .hysteresis
                .only_way_down;
            let speed_temperature_matrix = non_archived_argon_config.fan_config.matrix;
            set_fan_speed_loop(
                interval,
                hysteresis,
                only_way_down,
                SpeedTemperatureMatrix::NonArchivedMatrix(speed_temperature_matrix),
            );
        }
    }
}
