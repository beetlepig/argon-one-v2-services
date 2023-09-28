use rppal::gpio;
use rppal::i2c;

pub const DEVICE_ADDRESS: u16 = 0x1a;
pub const TURN_OFF_FAN: u8 = 0x00;
pub const CUT_POWER_BYTE: u8 = 0xff;

pub fn initialize_i2c() -> Result<i2c::I2c, i2c::Error> {
    let mut i2c = i2c::I2c::new()?;
    i2c.set_slave_address(DEVICE_ADDRESS)?;
    Ok(i2c)
}

pub fn initialize_gpio_pin() -> Result<gpio::InputPin, gpio::Error> {
    let gpio = gpio::Gpio::new()?;
    let mut pin = gpio.get(4)?.into_input_pulldown();
    pin.set_interrupt(gpio::Trigger::RisingEdge)?;
    Ok(pin)
}
