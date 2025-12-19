# 1 - Introduction

This repository contains examples of *low-level* platform-independent drivers for [STMicroelectronics](https://www.st.com/mems) sensors. Sensor drivers and examples are written in Rust programming language.

The STMicroelectronics naming convention for driver repositories is:
 - `PARTNUMBER-rs` (*e.g. lsm6dsv16x-rs*) for *low-level platform-independent drivers (PID)*

### 1.a - Repository structure

This repository is structed with a folder for each sensor driver, named `xxxxxxx-rs`, where `xxxxxxx` is the sensor part number.

Another folder, named  `util`, does not follow the above naming convention. It contains *other useful resources* such as libraries and crates. To `clone` the complete content of this folder, use the following command:

```git
git clone --recursive https://github.com/STMicroelectronics/st-mems-rust-drivers.git
```

### 1.b - Sensor driver folder structure

Every *sensor driver* folder contains the following:

- `xxxxxxx-rs` : This folder is hosted as a submodule repository and published as a standalone crate on the [crates.io](https://crates.io/). Documentation can be found on the corresponding [crates.io](https://crates.io/) page or generated locally using the command: `cargo doc`.
- `xxxxxxx-rs/examples`: This folder contains self-contained example projects to test the sensor. It may be necessary to modify the pin configuration or the I2C/SPI address as needed. The folder name of each examples includes the board used to test the sensor.
- `xxxxxxx-rs/README`: Contains additional info about the specific driver.

### 1.c - Getting started

A template is available in the `util` folder to help get started quickly with these drivers. The [cargo-generate](https://crates.io/crates/cargo-generate) tool may be used to configure a basic project environment by running:

```bash
cargo generate --git https://github.com/STMicroelectronics/st-mems-rust-drivers util/st-template
```

This template allows customization of the starting project by selecting the desired Nucleo board, framework ([Embassy](https://crates.io/crates/embassy-stm32) or [stm32-rs](https://github.com/stm32-rs)), and sensor. It also includes examples showing how to use I2C communication.

------

# 2 - Integration details

The driver is platform-independent. To use it on a general configuration, you need to:
- Set up the sensor hardware bus (e.g., SPI or I2C).
- Provide the configured bus instance to the sensor library.
- When necessary, configure the interrupt pin and implement platform-specific delay functions.

### 2.a Source code integration

Typically, the code can be used as presented in the example folder. However, to generalize the driver, a `BusOperation` trait is used. This allows for a generic bus that could be either I2C or SPI. The `util` folder wraps the trait in the [st-mems-bus](https://github.com/STMicroelectronics/st-mems-rust-drivers/tree/main/util/st-mems-bus) crate, enabling the same trait to be shared across all sensors and used simultaneously without redefining the trait. The configuration depends on the framework being used. Below is a minimal example with `sensorDriverCrate` referring to the specific driver crate and `SensorDriver` referring to the library's struct. Implementation for Embassy and STM32 frameworks are provided:

- **Embassy I2C**:
   ```rust
   use embassy_stm32::{bind_interrupts, Config};
   use embassy_stm32::dma::NoDma;
   use embassy_stm32::exti::ExtiInput;
   use embassy_stm32::gpio::{Input, Pull};
   use embassy_stm32::i2c::{self, Config as I2cConfig, I2c};
   use embassy_stm32::time::khz;
   use embassy_stm32::peripherals::{self, USART2};

   bind_interrupts!(struct Irqs {
      USART2 => BufferedInterruptHandler<USART2>;
      I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
      I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
   });

   // main section

   let p = embassy_stm32::init(Default::default());

   let i2c: I2c<_> = I2c::new(
      p.I2C1, // TBD: define the I2C channel as needed
      p.PB8, // TBD: define the scl route
      p.PB9, // TBD: define the sda route
      Irqs,
      NoDma, // TBD: provide Dma if available
      NoDma, // TBD: provide Dma if available
      khz(400),
      I2cConfig::default(),
   );

   let interrupt_pin = p.PC0; // TBD: define the interrupt pin accordingly
   let exti = p.EXTI0; // TBD: define the EXTI related to the interrupt pin
   let interrupt = Input::new(interrupt_pin, Pull::None);
   let mut interrupt = ExtiInput::new(interrupt, exti);

   let i2c_addr = sensorDriverCrate::I2CAddress::I2cAddH; // TBD: depends on whether SDA0 is high or not; see sensor's datasheet.

   let mut sensor = sensorDriverCrate::SensorDriver::new_i2c(i2c, i2c_addr).unwrap();
   ```

- **STM32 I2C**:
   ```rust
   use stm32f4xx_hal::{
      i2c::{DutyCycle, I2c, Mode},
      pac,
      prelude::*,
      serial::{config::Config, Serial},
   };

   // main section

   let dp = pac::Peripherals::take().unwrap();
   let cp = cortex_m::Peripherals::take().unwrap();

   let rcc = dp.RCC.constrain();
   let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();

   let gpiob = dp.GPIOB.split();
   let gpioa = dp.GPIOA.split();

   let scl = gpiob.pb8; // TBD: define the scl pin
   let sda = gpiob.pb9; // TBD: define the sda pin

   let i2c = I2c::new(
      dp.I2C1,
      (scl, sda),
      Mode::Standard {
         frequency: 400.kHz(),
      },
      &clocks,
   );

   let i2c_addr = sensorDriverCrate::I2CAddress::I2cAddH; // TBD: depends on whether SDA0 is high or not; see sensor's datasheet.

   let mut sensor = sensorDriverCrate::SensorDriver::new_i2c(i2c, i2c_addr).unwrap();
   ```

- **Embassy SPI**
   ```rust
   use core::cell::RefCell;
   use static_cell::StaticCell;
   use embassy_sync::blocking_mutex::NoopMutex;
   use embassy_time::{Delay, Duration, Timer, WithTimeout};

   use embassy_stm32 as hal;
   use hal::gpio::{Level, Output, Speed};
   use hal::sai;
   use hal::spi::Spi;
   use hal::{bind_interrupts, peripherals, usb};

   // main section

   let p = embassy_stm32::init(Default::default());

   let mut config = Config::default();
   config.mode = MODE_3; // TBD: define MODE
   config.frequency = embassy_stm32::time::Hertz(100_000); // TBD: define frequency
   let spi = Spi::new_blocking(p.SPI1, p.PA1, p.PA7, p.PA6, config); // TBD: define pins
   let bus = NoopMutex::new(RefCell::new(spi));
   let bus = SPI_BUS.init(bus);
   let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh); // TBD: define Chip select (CS) settings

   let spi = SpiDevice::new(bus, cs);
   let mut sensor = sensorDriverCrate::SensorDriver::new_spi(spi)
   ```

- **STM32 SPI**:
   ```rust
   use stm32f4xx_hal::spi::{Mode, NoMiso};
   use embedded_hal_bus::spi::ExclusiveDevice;
   use stm32f4xx_hal::{
      gpio::{self, Edge, Input},
      i2c::{I2c},
      spi::{Spi, Polarity, Phase},
      pac::{self, interrupt},
      prelude::*,
      serial::{config::Config, Serial},
   };

   // main section

   let dp = pac::Peripherals::take().unwrap();
   let cp = cortex_m::Peripherals::take().unwrap();

   let rcc = dp.RCC.constrain();
   let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();

   let gpiob = dp.GPIOB.split();
   let gpioa = dp.GPIOA.split();

   // SPI pins: SCK, MISO, MOSI
   let sck = gpioa.pa5.into_alternate();  // TBD:: define the sck pin
   let miso = gpioa.pa6.into_alternate(); // TBD:: define the miso pin
   let mosi = gpioa.pa7.into_alternate(); // TBD:: define the mosi pin

   let scl = gpiob.pb8; // TBD: define the scl pin
   let sda = gpiob.pb9; // TBD: define the sda pin

   // Chip Select (CS) pin
   let mut cs = gpiob.pb6.into_push_pull_output(); // TBD: define the gpio pin
   cs.set_high(); // Deselect by default

   let spi = Spi::new(
      dp.SPI1,                // TBD: define which SPIx to use
      (sck, miso, mosi),
      Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
      },
      2.MHz(),
      &clocks,
   );

   // Acquire SPI channel as Exclusive
   let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

   let mut sensor = sensorDriverCrate::SensorDriver::new_spi(spi_dev, tim1);
   ```

### 2.b Required properties

> * A rust compiler with a toolchain targeting the MCU.
> * Each sensor specifies a Minimum Supported Rust Version (MSRV) to ensure compatibility and successful compilation.

------

# 3 - Running examples

Examples are written for [STM32 Microcontrollers](https://www.st.com/en/microcontrollers.html) using the [NUCLEO_F401RE](https://github.com/STMicroelectronics/STMems_Standard_C_drivers/tree/master/_prj_NucleoF401) as primary platform. However, they can also serve as a guideline for every other platforms.

### 3.a Using STMicroelectronics evaluation boards

When using supported STMicroelectronics evaluation boards, the schematics provide information about which pins to use to setup the I2C or SPI communication with the sensor.

------

**More information: [http://www.st.com](http://st.com/MEMS)**

**Copyright (C) 2025 STMicroelectronics**