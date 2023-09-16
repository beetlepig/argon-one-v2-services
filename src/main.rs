use rppal::i2c::{Error, I2c};
use std::time::Duration;
use std::{env, thread};

const DEVICE_ADDRESS: u16 = 0x1a;
const TURN_OFF_FAN: u8 = 0x00;
const CUT_POWER_BYTE: u8 = 0xff;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args
        .iter()
        .find(|&arg| arg == "halt" || arg == "poweroff" || arg == "reboot" || arg == "kexec")
    {
        Some(arg) => {
            let init_i2c_result = init_i2c(arg);
            match init_i2c_result {
                Ok(_) => {
                    println!("Power off ran successfully")
                }
                Err(e) => {
                    eprintln!("Error with I2C: {}", e);
                }
            }
        }
        None => {
            eprintln!("No 'halt', 'poweroff', 'reboot' or 'kexec' arguments.");
        }
    }
}

fn init_i2c(arg: &str) -> Result<(), Error> {
    let mut i2c = I2c::new()?;
    i2c.set_slave_address(DEVICE_ADDRESS)?;

    println!("I2C Initialized");

    thread::sleep(Duration::from_millis(100));

    send_smbus_bytes(i2c, arg)
}

fn send_smbus_bytes(i2c: I2c, arg: &str) -> Result<(), Error> {
    // Turn off fan signal
    i2c.smbus_send_byte(TURN_OFF_FAN)?;

    thread::sleep(Duration::from_millis(100));

    // Power cut signal
    match arg {
        "halt" | "poweroff" => i2c.smbus_send_byte(CUT_POWER_BYTE),
        _ => Ok(()),
    }
}
