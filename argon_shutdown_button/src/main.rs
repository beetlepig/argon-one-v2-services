use shared_utils::initialize_device::initialize_gpio_pin;
use shared_utils::load_yaml::{load_argon_config, ArgonConfig, PowerScript};
use shared_utils::rppal::gpio::{InputPin, Level};
use std::process::{Child, Command};
use std::time::Duration;
use std::{fs, thread};

enum PowerOptions {
    Shutdown,
    Reboot,
}

fn main() {
    let argon_config = load_argon_config();
    let pin_result = initialize_gpio_pin();

    match pin_result {
        Ok(pin) => {
            println!("GPIO Pin 4 initialized successfully");
            wait_shutdown_button_interrupt(pin, &argon_config);
        }
        Err(e) => {
            eprintln!("Error initializing PIN 4: {}", e)
        }
    }

    println!("Shutdown button program finished");
}

fn wait_shutdown_button_interrupt(mut pin: InputPin, argon_config: &ArgonConfig) {
    let mut pulse_time: u16;
    let selected_power_option: PowerOptions;

    loop {
        pulse_time = 1;

        println!("Waiting shutdown button interrupt");

        let pool_interrupt_result = pin.poll_interrupt(true, None);
        match pool_interrupt_result {
            Ok(Some(Level::High)) => {
                thread::sleep(Duration::from_millis(10));
                while pin.is_high() {
                    thread::sleep(Duration::from_millis(10));
                    pulse_time = pulse_time + 1;
                }
            }
            Ok(Some(Level::Low)) => {
                eprintln!("Interrupt finished with Low level, this should not happen");
            }
            Ok(None) => {
                eprintln!("Interrupt finished with a None result, this should not happen");
            }
            Err(e) => {
                eprintln!("Interrupt Failed: {}", e);
            }
        }

        if pulse_time >= 2 && pulse_time <= 3 {
            println!("Starting reboot...");
            selected_power_option = PowerOptions::Reboot;
            break;
        } else if pulse_time >= 4 && pulse_time <= 5 {
            println!("Starting shutdown...");
            selected_power_option = PowerOptions::Shutdown;
            break;
        } else if pulse_time >= 6 && pulse_time <= 7 {
            println!("Starting forced shutdown...");
            selected_power_option = PowerOptions::Shutdown;
            break;
        }
    }

    run_shutdown_or_reboot_command(selected_power_option, &argon_config);
}

fn run_shutdown_or_reboot_command(power_option: PowerOptions, argon_config: &ArgonConfig) {
    let command_result = match power_option {
        PowerOptions::Shutdown => {
            run_power_command(&argon_config.shutdown_script, "shutdown", vec!["-h", "now"])
        }
        PowerOptions::Reboot => run_power_command(&argon_config.reboot_script, "reboot", vec![]),
    };

    match command_result {
        Ok(_) => {
            println!("Power command executed");
        }
        Err(e) => {
            eprintln!("Cannot run power command: {}", e);
        }
    }
}

fn run_power_command(
    script_config_option: &Option<PowerScript>,
    fallback_command: &str,
    fallback_args: Vec<&str>,
) -> std::io::Result<Child> {
    if let Some(script_config) = script_config_option {
        match fs::metadata(&script_config.location) {
            Ok(metadata) => {
                if metadata.is_file() {
                    return Command::new(&script_config.location)
                        .args(&script_config.args)
                        .spawn();
                }
            }
            Err(e) => {
                eprintln!("No a valid script: {}", e);
            }
        }
    }

    Command::new(fallback_command).args(fallback_args).spawn()
}
