use embedded_hal::spi::{SpiDevice, Operation};
use crate::BusOperation;

pub struct Spi<P> {
   pub spi: P
}

#[allow(dead_code)]
impl<P: SpiDevice> Spi<P> {
    /// Create new Spi instance
    ///
    /// # Arguments
    ///
    /// * `spi`: Instance of embedded hal SpiDevice
    ///
    /// # Returns
    ///
    /// * `Self`
    pub fn new(spi: P) -> Self {
        Self { spi }
    }
}

impl<P: SpiDevice> BusOperation for Spi<P> {
    type Error = P::Error;

    /// Reads bytes from the SPI bus.
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
        self.spi.transaction(&mut [Operation::Read(rbuf)])?;

        Ok(())
    }

    /// Writes bytes to the SPI bus.
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
        self.spi.transaction(&mut [Operation::Write(wbuf)])?;

        Ok(())
    }

    /// Writes a byte and then reads bytes from the SPI bus.
    ///
    /// # Arguments
    ///
    /// * `wbuf`: Buffer containing the byte to write.
    /// * `rbuf`: Buffer to store the read bytes.
    ///
    /// # Returns
    ///
    /// * `Result`
    ///      * `()`
    ///     * `Err`: Returns an error if the write-read operation fails.
    #[inline]
    fn write_byte_read_bytes(
        &mut self,
        wbuf: &[u8; 1],
        rbuf: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.spi
            .transaction(&mut [Operation::Write(&[wbuf[0] | 0x80]), Operation::Read(rbuf)])?;

        Ok(())
    }   
}
