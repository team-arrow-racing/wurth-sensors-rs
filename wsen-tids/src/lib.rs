#![no_std]

use embedded_hal::i2c::{I2c, SevenBitAddress};

/// I²C device address selection
#[derive(Copy, Clone)]
pub enum AddressSelect {
    High = 0b0111000,
    Low = 0b0111111,
}

impl Into<SevenBitAddress> for AddressSelect {
    fn into(self) -> SevenBitAddress {
        self as u8
    }
}

// register offsets
const REG_DEVICE_ID: u8 = 0x01;
const REG_TEMP_HIGH_LIMIT: u8 = 0x02;
const REG_TEMP_LOW_LIMIT: u8 = 0x03;
const REG_CONTROL: u8 = 0x04;
const REG_STATUS: u8 = 0x05;
const REG_DATA_TEMP_L: u8 = 0x06;
const REG_DATA_TEMP_H: u8 = 0x07;
const REG_SOFT_RESET: u8 = 0x0C;

/// Continuous conversion speed
pub enum Speed {
    Hz25 = 0b00,
    Hz50 = 0b01,
    Hz100 = 0b10,
    Hz200 = 0b11,
}

/// Sensor operating mode
pub enum Mode {
    PowerDown,
    SingleConversion,
    Continuous(Speed),
}

pub struct Sensor<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> Sensor<I2C> {
    /// Creates a new sensor instance.
    pub fn new(i2c: I2C, address: AddressSelect) -> Self {
        Self {
            i2c,
            address: address.into(),
        }
    }

    /// Read device ID from the sensor.
    ///
    /// This is fixed number (0xA0).
    pub fn read_device_id(&mut self) -> Result<u8, I2C::Error> {
        let mut buf: [u8; 1] = [0];

        match self.i2c.read(self.address + REG_DEVICE_ID, &mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(e) => Err(e),
        }
    }

    /// Disable high temperature limit interrupt generation.
    pub fn disable_temperature_high_limit(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(self.address + REG_TEMP_HIGH_LIMIT, &[0])
    }

    /// Disable low temperature limit interrupt generation.
    pub fn disable_temperature_low_limit(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(self.address + REG_TEMP_LOW_LIMIT, &[0])
    }

    /// Sets the temperature threshold high limit in degrees celcius.
    pub fn temperature_high_limit(&mut self, celcius: f32) -> Result<(), I2C::Error> {
        let value = temperature_to_reg_value(celcius);

        self.i2c.write(self.address + REG_TEMP_HIGH_LIMIT, &[value])
    }

    /// Sets the temperature threshold high limit in degrees celcius.
    pub fn temperature_low_limit(&mut self, celcius: f32) -> Result<(), I2C::Error> {
        let value = temperature_to_reg_value(celcius);

        self.i2c.write(self.address + REG_TEMP_LOW_LIMIT, &[value])
    }

    pub fn configure(&mut self, mode: Mode) -> Result<(), I2C::Error> {
        todo!("Implement register configuration");
        let value = match mode {
            Mode::PowerDown => 0,
            Mode::SingleConversion => 0,
            Mode::Continuous(_) => 0,
        };

        self.i2c.write(self.address + REG_CONTROL, &[value])
    }

    /// Read the temperature from the sensor.
    pub fn read_temperature(&mut self) -> Result<f32, I2C::Error> {
        let mut buf: [u8; 1] = [0];

        let low: u16 = match self.i2c.read(self.address + REG_DATA_TEMP_L, &mut buf) {
            Ok(_) => buf[0] as u16,
            Err(e) => return Err(e),
        };

        let mut buf: [u8; 1] = [0];

        let high: u16 = match self.i2c.read(self.address + REG_DATA_TEMP_H, &mut buf) {
            Ok(_) => buf[0] as u16,
            Err(e) => return Err(e),
        };

        let composite: f32 = (high << 8 | low) as f32;

        Ok(composite * 0.01)
    }

    /// Perform a software reset of the sensor.
    ///
    /// Resets all digital blocks.
    pub fn reset(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(self.address + REG_SOFT_RESET, &[1 << 1])
    }
}

/// Converts a floating-point temperature into the required register value.
///
/// See table 10 in the user manual for more details.
fn temperature_to_reg_value(celcius: f32) -> u8 {
    ((celcius / 0.64) + 63.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_conversion() {
        // examples copied from table 10 in reference manual
        // rounded towards zero by 0.001 to work properly for some cases
        assert_eq!(temperature_to_reg_value(-39.68), 1);
        assert_eq!(temperature_to_reg_value(-39.04 + 0.001), 2);
        assert_eq!(temperature_to_reg_value(-38.40 + 0.001), 3);
        // ...
        assert_eq!(temperature_to_reg_value(-0.64), 62);
        assert_eq!(temperature_to_reg_value(0.0), 63);
        assert_eq!(temperature_to_reg_value(0.64), 64);
        // ...
        assert_eq!(temperature_to_reg_value(122.24), 254);
        assert_eq!(temperature_to_reg_value(122.88), 255);
    }
}
