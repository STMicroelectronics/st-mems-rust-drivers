//! I2C example
//!
//! This example demonstrates I2C communication
//!
//! The reference board used is the NUCLEO-F401RE; other boards may require
//! setting the pins accordingly.
//!
//! Default pins: PB8 (SCL), PB9 (SDA)

#![no_std]
#![no_main]

{% if framework == "stm32rs" -%}
use defmt::*;
use {defmt_rtt as _, panic_probe as _};
use cortex_m_rt::entry;
use stm32f4xx_hal::{
    pac,
    i2c::I2c,
    prelude::*,
};


#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.MHz()).sysclk(48.MHz()).freeze();

    let gpiob = dp.GPIOB.split();

    // Configure I2C pins
    let scl = gpiob.pb8.into_alternate().set_open_drain();
    let sda = gpiob.pb9.into_alternate().set_open_drain();

    // Create I2C interface
    let mut i2c = I2c::new(dp.I2C1, (scl, sda), 400.kHz(), &clocks);

    // Wait a boot time
    let mut delay = cp.SYST.delay(&clocks);
    delay.delay_ms(5);

    // Example: Scan for I2C devices
    for addr in 0x08..0x78 {
        if i2c.write(addr, &[]).is_ok() {
            // Device found at address
            // In a real application, you would handle this
            info!("Device found at addr: 0x{:02X}", addr);
        }
    }

    loop {}
}
{% endif -%}

{% if framework == "embassy" -%}
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    i2c::{self, I2c, Config as I2cConfig},
    time::khz,
    peripherals,
    dma::NoDma,
    bind_interrupts
};
use {defmt_rtt as _, panic_halt as _};

#[defmt::panic_handler]
fn panic() -> ! {
    core::panic!("panic via `defmt::panic!`")
}

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("I2C example starting...");

    let p = embassy_stm32::init(Default::default());

    let mut i2c: I2c<_> = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        NoDma,
        NoDma,
        khz(100),
        I2cConfig::default(),
    );

    info!("Scanning I2C bus...");

    // Example: Scan for I2C devices
    for addr in 0x08..0x78_u8 {
        match i2c.blocking_write(addr, &[]) {
            Ok(_) => info!("Device found at address 0x{:02X}", addr),
            Err(_) => {}, // No device at this address
        }
    }

    info!("Search has ended");
}
{% endif -%}
