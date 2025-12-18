//! Blink example
//!
//! This example blink a led on a board
//!
//! The reference board used is the NUCLEO-F401RE; other boards may require
//! setting the pins accordingly.
//!
//! Default pins: PA5 (LED)

#![no_std]
#![no_main]

{% if framework == "stm32rs" -%}
use {defmt_rtt as _, panic_probe as _};
use cortex_m_rt::entry;
use stm32f4xx_hal::{
    pac,
    prelude::*,
};
use defmt::*;

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.MHz()).sysclk(48.MHz()).freeze();

    // Acquire the GPIO peripheral
    let gpioa = dp.GPIOA.split();

    // Configure PA5 as a push-pull output
    let mut led = gpioa.pa5.into_push_pull_output();

    // Create a delay abstraction based on SysTick
    let mut delay = cp.SYST.delay(&clocks);
    delay.delay_ms(5);

    info!("Start toggling the led");

    loop {
        // Toggle the LED
        led.toggle();

        // Wait for 100 ms
        delay.delay_ms(100);
    }
}
{% endif -%}

{% if framework == "embassy" -%}
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    // Configure PA5 as output
    let mut led = Output::new(p.PA5, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
{% endif -%}
