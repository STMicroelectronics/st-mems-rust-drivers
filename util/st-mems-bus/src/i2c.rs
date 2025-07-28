use embedded_hal::i2c::{I2c, SevenBitAddress};
use crate::BusOperation;

pub struct I2cBus<T: I2c> {
    pub i2c: T,
    pub address: SevenBitAddress
}

#[allow(dead_code)]
impl<T: I2c> I2cBus<T> {
    /// Create new I2C instance
    ///
    /// # Arguments
    ///
    /// * `i2c`: Instance of embedded hal I2c
    /// * `address`: Address of the i2c
    ///
    /// # Returns
    ///
    /// * `Self`
    pub fn new(i2c: T, address: SevenBitAddress) -> Self {
        Self { i2c, address }
    }
}

impl<T: I2c> BusOperation for I2cBus<T> {
    type Error = T::Error;

    /// Reads bytes from the I2C bus.
    ///
    /// # Arguments
    ///
    /// * `rbuf`: Buffer to store the read bytes.
    ///
    /// # Returns
    ///
    /// * `Result`
    ///     * `()`
    ///     * `Err`: Returns an error if the read operation fails.
    #[inline]
    fn read_bytes(&mut self, rbuf: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.read(self.address, rbuf)?;

        Ok(())
    }

    /// Writes bytes to the I2C bus.
    ///
    /// # Arguments
    ///
    /// * `wbuf`: Buffer containing the bytes to write.
    ///
    /// # Returns
    ///
    /// * `Result`
    ///     * `()`
    ///     * `Err`: Returns an error if the write operation fails.
    #[inline]
    fn write_bytes(&mut self, wbuf: &[u8]) -> Result<(), Self::Error> {
        self.i2c.write(self.address, wbuf)?;

        Ok(())
    }

    /// Writes a byte and then reads bytes from the I2C bus.
    ///
    /// # Arguments
    ///
    /// * `wbuf`: Buffer containing the byte to write.
    /// * `rbuf`: Buffer to store the read bytes.
    ///
    /// # Returns
    ///
    /// * `Result`
    ///     * `()`
    ///     * `Err`: Returns an error if the write-read operation fails.
    #[inline]
    fn write_byte_read_bytes(
        &mut self,
        wbuf: &[u8; 1],
        rbuf: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c.write_read(self.address, wbuf, rbuf)?;

        Ok(())
    }

}
