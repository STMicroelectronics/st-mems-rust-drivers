#![no_std]
use core::fmt::Debug;
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

#[cfg(feature = "i2c")]
pub mod i2c;
#[cfg(feature = "spi")]
pub mod spi;

const CHUNK_SIZE: usize = 255;

pub trait BusOperation {
    type Error: Debug;

    fn read_bytes(&mut self, rbuf: &mut [u8]) -> Result<(), Self::Error>;
    fn write_bytes(&mut self, wbuf: &[u8]) -> Result<(), Self::Error>;
    fn write_byte_read_bytes(&mut self, wbuf: &[u8; 1], rbuf: &mut [u8])-> Result<(), Self::Error>;

    #[inline]
    fn read_from_register(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.write_byte_read_bytes(&[reg], buf)
    }
    #[inline]
    fn write_to_register(&mut self, reg: u8, buf: &[u8]) -> Result<(), Self::Error> {
        let size = buf.len();
        let mut write_size: usize;
        let mut tmp: [u8; CHUNK_SIZE + 1] = [0; CHUNK_SIZE + 1];
        for i in (0..size).step_by(CHUNK_SIZE - 1) {
            write_size = if size - i >= CHUNK_SIZE - 1 {
                CHUNK_SIZE - 1
            } else {
                size - i
            };
            tmp[0] = reg + (i / (CHUNK_SIZE - 1)) as u8;
            tmp[1..(write_size + 1)].copy_from_slice(&buf[i..(write_size + i)]);
            self.write_bytes(&tmp[..1 + write_size])?;
        }
        Ok(())
    }
}

pub trait MemBankFunctions<M> {
    type Error;    

    fn mem_bank_set(&mut self, val: M) -> Result<(), Self::Error>;
    fn mem_bank_get(&mut self) -> Result<M, Self::Error>;
}

pub trait EmbAdvFunctions {
    type Error;    

    fn ln_pg_write(
        &mut self,
        address: u16,
        buf: &[u8],
        len: u8,
    ) -> Result<(), Self::Error>;

    fn ln_pg_read(
        &mut self,
        address: u16,
        buf: &mut [u8],
        len: u8,
    ) -> Result<(), Self::Error>;
    
}

pub struct Owned<P> {
    pub value: P
}

impl<P> Owned<P> {
    pub fn new(value: P) -> Self {
        Self { value }
    }
}

pub struct Shared<'a, P> {
    pub value: &'a RefCell<P>
}

impl<'a, P> Shared<'a, P> {
    pub fn new(value: &'a RefCell<P>) -> Self {
        Self { value }
    }
}

impl<P> BusOperation for Owned<P> where P: BusOperation {
    type Error = P::Error;
    
    /// Reads bytes from the bus.
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
        self.value.read_bytes(rbuf)
    }

    /// Writes bytes to the bus.
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
        self.value.write_bytes(wbuf)
    }
    
    /// Writes a byte and then reads bytes from the bus.
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
    fn write_byte_read_bytes(&mut self, wbuf: &[u8; 1], rbuf: &mut [u8])-> Result<(), Self::Error> {
        self.value.write_byte_read_bytes(wbuf, rbuf)
    }
}

impl<'a, P> BusOperation for Shared<'a, P> where P: BusOperation {
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
        self.value.borrow_mut().read_bytes(rbuf)?;

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
        self.value.borrow_mut().write_bytes(wbuf)?;
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
        self.value.borrow_mut()
            .write_byte_read_bytes(wbuf, rbuf)?;

        Ok(())
    }
}

impl<'a, P> DelayNs for Shared<'a, P> where P: DelayNs {
    fn delay_ms(&mut self, ms: u32) {
        self.value.borrow_mut().delay_ms(ms)
    }

    fn delay_ns(&mut self, ns: u32) {
        self.value.borrow_mut().delay_ns(ns);
    }

    fn delay_us(&mut self, us: u32) {
        self.value.borrow_mut().delay_us(us);
    }
}

impl<'a, P> embedded_hal::i2c::ErrorType for Shared<'a, P>
where
    P: I2c,
{
    type Error = P::Error;
}

impl<'a, P> I2c for Shared<'a, P> where P: I2c {

    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.value.borrow_mut().read(address, read)
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.value.borrow_mut().write(address, write)
    }

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.value.borrow_mut().write_read(address, write, read)
    }

    fn transaction(
            &mut self,
            address: u8,
            operations: &mut [embedded_hal::i2c::Operation<'_>],
        ) -> Result<(), Self::Error> {
        self.value.borrow_mut().transaction(address, operations)
    }
    
}
