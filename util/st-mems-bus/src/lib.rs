#![no_std]

#[cfg(feature = "is_sync")]
use embedded_hal::{
    i2c::{I2c, SevenBitAddress},
    spi::{Operation, SpiDevice},
};

#[cfg(not(feature = "is_sync"))]
use embedded_hal_async::{
    i2c::{I2c, SevenBitAddress},
    spi::{Operation, SpiDevice},
};

#[cfg(feature = "i2c")]
pub mod i2c;
pub mod ops;
#[cfg(feature = "spi")]
pub mod spi;

pub use i2c::*;
pub use ops::*;

const CHUNK_SIZE: usize = 256;
