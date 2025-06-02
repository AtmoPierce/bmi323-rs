use crate::Error;
// use embedded_hal_async::i2c;

// /// I2C communication interface for BMI323
// #[derive(Debug)]
// pub struct AsyncI2cInterface<I2C>{
//     pub(crate) i2c: I2C,
//     pub(crate) address: u8,
// }

// /// Trait for writing data to the BMI323
// pub trait AsyncWriteData {
//     type Error;
//     /// Write a single byte to a register
//     ///
//     /// # Arguments
//     ///
//     /// * `register` - The register address
//     /// * `data` - The byte to write
//     async fn write_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error>;
//     /// Write multiple bytes of data
//     ///
//     /// # Arguments
//     ///
//     /// * `payload` - The data to write
//     async fn write_data(&mut self, payload: &[u8]) -> Result<(), Self::Error>;
// }

// impl<I2C, E> AsyncWriteData for AsyncI2cInterface<I2C>
// where
//     I2C: i2c::I2c<Error = E>,{
//     type Error = Error<E>;
//     async fn write_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error> {
//         let payload: [u8; 2] = [register, data];
//         self.i2c.write(self.address, &payload).await.map_err(Error::Comm)
//     }

//     async fn write_data(&mut self, payload: &[u8]) -> Result<(), Self::Error> {
//         self.i2c.write(self.address, payload).await.map_err(Error::Comm)
//     }
// }


// pub trait AsyncReadData {
//     type Error;
//     /// Read a single byte from a register
//     ///
//     /// # Arguments
//     ///
//     /// * `register` - The register address to read from
//     async fn read_register(&mut self, register: u8) -> Result<u8, Self::Error>;
//     /// Read multiple bytes of data
//     ///
//     /// # Arguments
//     ///
//     /// * `payload` - Buffer to store the read data
//     async fn read_data<'a>(&mut self, payload: &'a mut [u8]) -> Result<&'a [u8], Self::Error> ;
// }
// impl<I2C, E> AsyncReadData for AsyncI2cInterface<I2C>
// where I2C: i2c::I2c<Error = E>,{
//     type Error = Error<E>;
//     async fn read_register(&mut self, register: u8) -> Result<u8, Self::Error> {
//         let mut temp_data = [0u8; 128];
//         let mut data = [0u8; 2];
//         self.i2c
//             .write_read(self.address, &[register], &mut temp_data).await
//             .map_err(Error::Comm)?;
//         for i in 0..data.len() {
//             data[i] = temp_data[i + 2];
//         }
//         Ok(data[0])
//     }

//     async fn read_data<'a>(&mut self, payload: &'a mut [u8]) -> Result<&'a [u8], Error<E>> {
//         let address = payload[0];
//         let len = payload.len();
//         let data = &mut payload[1..len];

//         let total_len = data.len() + 2;
//         let mut temp_buf = [0u8; 128]; // Temporary buffer to hold dummy bytes and data

//         self.i2c
//             .write_read(self.address, &[address], &mut temp_buf[..total_len]).await
//             .map_err(Error::Comm)?;

//         // Copy data from temp_buf to data, skipping dummy bytes
//         data.copy_from_slice(&temp_buf[2..total_len]);

//         Ok(data)
//     }
// }