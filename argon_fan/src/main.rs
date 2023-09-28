mod mapper;

use crate::mapper::matrix_mapper;
use rustc_hash::FxHashMap;
use shared_utils::initialize_device::initialize_i2c;
use shared_utils::load_yaml::load_argon_config;
use std::thread;
use std::time::Duration;
use systemstat::Platform;

fn main() {
    let loaded_argon_config = load_argon_config();
    let interval = Duration::from_millis(loaded_argon_config.fan_config.interval as u64);
    let hysteresis = loaded_argon_config.fan_config.hysteresis.amount;
    let only_way_down = loaded_argon_config.fan_config.hysteresis.only_way_down;
    let mapped_temperature_matrix = matrix_mapper(&loaded_argon_config.fan_config.matrix);

    let mut retries: u8 = 0;
    let mut last_temperature: f32 = 0.0;

    while retries <= 3 {
        match control_fan(
            &mapped_temperature_matrix,
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

fn control_fan(
    mapped_temperature_matrix: &FxHashMap<u8, u8>,
    hysteresis: u8,
    last_temperature: f32,
    only_way_down: bool,
) -> Result<f32, Box<dyn std::error::Error>> {
    let i2c = initialize_i2c()?;
    let sys = systemstat::System::new();
    let device_temperature: f32 = sys.cpu_temp()?;
    let temperature_delta: f32 = device_temperature - last_temperature;

    println!(
        "Last target temperature: {}. Current temperature: {}. Delta: {}",
        last_temperature, device_temperature, temperature_delta
    );

    if only_way_down && temperature_delta > 0.0 || temperature_delta.abs() > hysteresis as f32 {
        if let Some(&fan_speed) = mapped_temperature_matrix.get(&(device_temperature.round() as u8))
        {
            println!("Set new fan speed to device: {}", fan_speed);
            i2c.smbus_send_byte(fan_speed)?;
            print!("\n");
            return Ok(device_temperature);
        }
    }

    Ok(last_temperature)
}
