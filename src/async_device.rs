use crate::{
    // async_interface::{AsyncI2cInterface, AsyncReadData, AsyncWriteData},
    types::{AccelerometerRange, GyroscopeRange, Sensor3DData, Sensor3DDataScaled, SensorType},
    AccelConfig, AsyncBmi323, GyroConfig, Register,
};
#[cfg(feature = "defmt")]
use defmt::{error, info};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

#[cfg(feature = "defmt")]
fn log_bmi_info(message: &'static str) {
    info!("{}", message);
}

#[cfg(not(feature = "defmt"))]
fn log_bmi_info(_message: &'static str) {}

#[cfg(feature = "defmt")]
fn log_bmi_register(label: &'static str, value: u8) {
    info!("{}: {}", label, value);
}

#[cfg(not(feature = "defmt"))]
fn log_bmi_register(_label: &'static str, _value: u8) {}

#[cfg(feature = "defmt")]
fn log_bmi_i2c_error(context: &'static str) {
    error!("BMI323 I2C communication error during {}", context);
}

#[cfg(not(feature = "defmt"))]
fn log_bmi_i2c_error(_context: &'static str) {}

impl<I2C, D> AsyncBmi323<I2C, D> where D: DelayNs{
    /// Create a new BMI323 device instance
    ///
    /// # Arguments
    ///
    /// * `iface` - The communication interface
    /// * `delay` - A delay provider
    pub fn new_with_i2c(i2c: I2C, address: u8, delay: D) -> Self {
        AsyncBmi323 {
            i2c,
            address,
            delay,
            accel_range: AccelerometerRange::default(),
            gyro_range: GyroscopeRange::default(),
        }
    }
}

impl<I2C, D> AsyncBmi323<I2C, D> where I2C: I2c, D: DelayNs{
    /// Initialize the device
    pub async fn init(&mut self) -> Result<i8, I2C::Error> {
        let soft_reset_result = self.write_register_16bit(Register::CMD, Register::CMD_SOFT_RESET).await;
        match soft_reset_result{
            Ok(_)=>{
                log_bmi_info("Soft Reset Sent");
            }
            Err(i2c_error) => {
                log_bmi_i2c_error("soft reset");
                return Err(i2c_error);
            }
        }

        self.delay.delay_us(2000).await;
        let mut result = 0;
        let status_result = self.read_register(0x01).await;
        match status_result{
            Ok(data)=>{
                log_bmi_register("Status", data);
                if (data & 0b0000_0001) != 0 {
                    result = -1;
                }
                else{
                    result = 0;
                }
            }
            Err(i2c_error)=>{
                log_bmi_i2c_error("status read");
                return Err(i2c_error);
            }
        }
        let id_result = self.read_register(Register::CHIPID).await;
        match id_result{
            Ok(data)=>{
                log_bmi_register("ID", data);
                if data != Register::BMI323_CHIP_ID {
                    result = -1;
                    return Ok(result);
                }
                else{
                    Ok(result)
                }                               
            }
            Err(i2c_error)=>{
                log_bmi_i2c_error("chip id read");
                return Err(i2c_error);
            }            
        }
    }

    /// Set the accelerometer configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The accelerometer configuration
    pub async fn set_accel_config(&mut self, config: AccelConfig) -> Result<(), I2C::Error> {
        let reg_data = self.config_to_reg_data(config).await;
        self.write_register_16bit(Register::ACC_CONF, reg_data).await?;
        self.accel_range = config.range;

        // Wait for accelerometer data to be ready
        self.wait_for_data_ready(SensorType::Accelerometer).await;

        Ok(())
    }

    /// Set the gyroscope configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The gyroscope configuration
    pub async fn set_gyro_config(&mut self, config: GyroConfig) -> Result<(), I2C::Error> {
        let reg_data = self.config_to_reg_data(config).await;
        self.write_register_16bit(Register::GYR_CONF, reg_data).await?;
        self.gyro_range = config.range;

        // Wait for gyroscope data to be ready
        self.wait_for_data_ready(SensorType::Gyroscope).await?;

        Ok(())
    }

    async fn config_to_reg_data<T>(&self, config: T) -> u16
    where
        T: Into<u16> + Copy,
    {
        let config: u16 = config.into();
        config
    }

    async fn read_sensor_data(&mut self, sensor_type: SensorType) -> Result<Sensor3DData, I2C::Error> {
        let (base_reg, data_size) = match sensor_type {
            SensorType::Accelerometer => (Register::ACC_DATA_X, 21),
            SensorType::Gyroscope => (Register::GYR_DATA_X, 15),
        };

        let mut data = [0u8; 21]; // Use the larger size
        data[0] = base_reg;
        let sensor_data = self.read_data(&mut data[0..data_size]).await?;

        Ok(Sensor3DData {
            x: i16::from_le_bytes([sensor_data[0], sensor_data[1]]),
            y: i16::from_le_bytes([sensor_data[2], sensor_data[3]]),
            z: i16::from_le_bytes([sensor_data[4], sensor_data[5]]),
        })
    }

    /// Read the LSB for the accelerometer
    pub async fn read_accel_data(&mut self) -> Result<Sensor3DData, I2C::Error> {
        self.read_sensor_data(SensorType::Accelerometer).await
    }

    /// Read the LSB for the gyroscope
    pub async fn read_gyro_data(&mut self) -> Result<Sensor3DData, I2C::Error> {
        self.read_sensor_data(SensorType::Gyroscope).await
    }

    /// Read the LSB for the accelerometer and return the scaled value as mps2
    pub async fn read_accel_data_scaled(&mut self) -> Result<Sensor3DDataScaled, I2C::Error> {
        self.read_accel_data()
            .await
            .map(|raw_data| raw_data.to_mps2(self.accel_range.to_g())) // Assuming 16-bit width
    }

    /// Read the LSB for the gyroscope and return the scaled value as dps
    pub async fn read_gyro_data_scaled(&mut self) -> Result<Sensor3DDataScaled, I2C::Error> {
        self.read_gyro_data()
            .await
            .map(|raw_data| raw_data.to_dps(self.gyro_range.to_dps())) // Assuming 16-bit width
    }

    async fn write_register_16bit(&mut self, reg: u8, value: u16) -> Result<(), I2C::Error> {
        let bytes = value.to_le_bytes();
        self.write_data(&[reg, bytes[0], bytes[1]]).await
    }
 
    async fn write_register(&mut self, register: u8, data: u8) -> Result<(), I2C::Error> {
        let payload: [u8; 2] = [register, data];
        self.i2c.write(self.address, &payload).await
    }

    async fn write_data(&mut self, payload: &[u8]) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, payload).await
    }

    async fn read_register(&mut self, reg: u8) -> Result<u8, I2C::Error> {
        let mut temp_data = [0u8; 128];
        let mut data = [0u8; 2];
        let result = self.i2c.write_read(self.address, &[reg], &mut temp_data).await;
        for i in 0..data.len() {
            data[i] = temp_data[i+2];
        }
        return Ok(data[0]);
    }

    async fn read_data<'a>(&mut self, payload: &'a mut [u8]) -> Result<&'a [u8], I2C::Error> {
        let address = payload[0];
        let write_addresss = [address];
        let len = payload.len();
        let data = &mut payload[1..len];

        let total_len = data.len() + 2;
        let mut temp_buf = [0u8; 128]; // Temporary buffer to hold dummy bytes and data

        let data_result = self.i2c
            .write_read(self.address, &write_addresss, &mut temp_buf[..total_len]).await;
        // Copy data from temp_buf to data, skipping dummy bytes
        data.copy_from_slice(&temp_buf[2..total_len]);
        Ok(data)
    }

    async fn wait_for_data_ready(&mut self, sensor_type: SensorType) -> Result<(), I2C::Error> {
        const MAX_RETRIES: u8 = 100;
        let mut retries = 0;

        while !self.is_data_ready(sensor_type).await? {
            if retries >= MAX_RETRIES {
                return Ok(());
                // return Err();
            }
            self.delay.delay_ms(1).await;
            retries += 1;
        }

        Ok(())
    }

    async fn is_data_ready(&mut self, sensor_type: SensorType) -> Result<bool, I2C::Error> {
        let status = self.read_register(Register::STATUS).await?;
        match sensor_type {
            SensorType::Accelerometer => Ok((status & 0b1000_0000) != 0), // Check bit 7 (drdy_acc)
            SensorType::Gyroscope => Ok((status & 0b0100_0000) != 0),     // Check bit 6 (drdy_gyr)
        }
    }
}
