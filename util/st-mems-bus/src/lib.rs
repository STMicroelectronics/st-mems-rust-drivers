#![no_std]

#[path = "."]
pub mod asynchronous {
    use bisync::asynchronous::*;
    use embedded_hal_async::i2c::{I2c, SevenBitAddress};
    use embedded_hal_async::spi::{Operation, SpiDevice};

    #[cfg(feature = "i2c")]
    pub mod i2c;
    pub mod ops;
    #[cfg(feature = "spi")]
    pub mod spi;

    #[cfg(feature = "i2c")]
    pub use i2c::*;
    pub use ops::*;
    #[cfg(feature = "spi")]
    pub use spi::*;
}

#[path = "."]
pub mod blocking {
    use bisync::synchronous::*;
    use embedded_hal::i2c::{I2c, SevenBitAddress};
    use embedded_hal::spi::{Operation, SpiDevice};

    #[cfg(feature = "i2c")]
    pub mod i2c;
    pub mod ops;
    #[cfg(feature = "spi")]
    pub mod spi;

    pub use i2c::*;
    pub use ops::*;
}

const CHUNK_SIZE: usize = 256;
