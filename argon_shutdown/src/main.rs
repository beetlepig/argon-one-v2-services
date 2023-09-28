use shared_utils::initialize_device::{initialize_i2c, CUT_POWER_BYTE, TURN_OFF_FAN};
use shared_utils::rppal::i2c::{Error, I2c};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args
        .iter()
        .find(|&arg| arg == "halt" || arg == "poweroff" || arg == "reboot" || arg == "kexec")
    {
        Some(arg) => match send_smbus_bytes(initialize_i2c(), arg) {
            Ok(_) => {
                println!("Power off ran successfully")
            }
            Err(e) => {
                eprintln!("Error with I2C: {}", e);
            }
        },
        None => {
            eprintln!("No 'halt', 'poweroff', 'reboot' or 'kexec' arguments.");
        }
    }
}

fn send_smbus_bytes(i2c_result: Result<I2c, Error>, arg: &str) -> Result<(), Error> {
    let i2c = i2c_result?;
    // Turn off fan signal
    i2c.smbus_send_byte(TURN_OFF_FAN)?;

    // Power cut signal
    match arg {
        "halt" | "poweroff" => i2c.smbus_send_byte(CUT_POWER_BYTE),
        _ => Ok(()),
    }
}
