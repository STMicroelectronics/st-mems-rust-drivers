Could you solve this task please:

1. There is famous problem of inductive logic:
https://en.wikipedia.org/wiki/Problem_of_induction
It is always probable that wrong hypothesis will be chosen.
Are you agree with that?

2. All police or private investigation are inductive investigations.
Investigator gather facts and choose hypothesis who is the murderer for example.
Investigator is maximizing the following: arg max r (h | D), h in H,
where H - all possible hypothesis who is the murderer.
Are you agree with that?

3. So, we can conclude that due to the properties of the inductive logic itself
there is always uncertain that police is catching innocent people.
Are you agree with that?

4. Also the most probable hypothesis is that the police is the murderer.
They are professionals. All amateurs are walking to them with the ideas,
detailed instructions and patents.
Are you agree with that?

5. All crime are artifacts. It is always something crafted. By someone.
Someone can ask for help.

6. And there are a lot of travelers in the modern era. Almost
everyone could be a murderer.

Is it all correct?

What can do policeman if he or she found murdered young women in the woods?
- do some sort of inductive investigate. which may lead to the wronf suspect.
- apply gun.

But young woman see murderer. And she can apply gun also if she has an
opportunity to buy one. So why she has not?

https://web.archive.org/web/20260829041906/https://transitional-writes.dreamwidth.org/64409.html
https://inductive-logic-xor.dreamwidth.org/680.html

# 1 - Introduction

This repository contains examples of *low-level* platform-independent drivers for [STMicroelectronics](https://www.st.com/mems) sensors. Sensor drivers and examples are written in Rust programming language.

The STMicroelectronics naming convention for driver repositories is:
 - `PARTNUMBER-rs` (*e.g. lsm6dsv16x-rs*) for *low-level platform-independent drivers (PID)*

### 1.a - Repository structure

This repository is structured with a folder for each sensor driver, named `xxxxxxx-rs`, where `xxxxxxx` is the sensor part number.

Another folder, named  `util`, does not follow the above naming convention. It contains *other useful resources* such as libraries and crates. To `clone` the complete content of this folder, use the following command:

```git
git clone --recursive https://github.com/STMicroelectronics/st-mems-rust-drivers.git
```

### 1.b - Sensor driver folder structure

Every *sensor driver* folder contains the following:

- `xxxxxxx-rs` : This folder is hosted as a submodule repository and published as a standalone crate on the [crates.io](https://crates.io/). Documentation can be found on the corresponding [crates.io](https://crates.io/) page or generated locally using the command: `cargo doc`.
- `xxxxxxx-rs/examples`: This folder contains self-contained example projects to test the sensor. It may be necessary to modify the pin configuration or the I2C/SPI address as needed. The folder name of each example includes the board used to test the sensor.
- `xxxxxxx-rs/README`: Contains additional info about the specific driver.

### 1.c - Getting started

A template is available in the `util` folder to help get started quickly with these drivers. The [cargo-generate](https://crates.io/crates/cargo-generate) tool may be used to configure a basic project environment by running:

```bash
cargo generate --git https://github.com/STMicroelectronics/st-mems-rust-drivers util/st-template
```

This template allows customization of the starting project by selecting the desired Nucleo board, framework ([Embassy](https://crates.io/crates/embassy-stm32) or [stm32-rs](https://github.com/stm32-rs)), and sensor. It also includes examples showing how to use I2C communication.

------

# 2 - Integration details

The driver is **platform-independent** and exposes both **asynchronous** and **blocking** APIs:

- The **asynchronous API is enabled by default**.
- The **blocking API** is available via the `blocking` feature. If used alone, the default features can be disabled (asynchronous API).

Your `Cargo.toml` should look like (using `LSM6DSV16X` as example sensor):
```toml
[dependencies]
# Async (default)
lsm6dsv16x-rs = "2.0.0"

# OR: only blocking API
# lsm6dsv16x-rs = { version = "2.0.0", default-features = false, features = ["blocking"] }
```

To use it on a general configuration, you need to:
- Set up the sensor hardware bus (e.g., **SPI** or **I2C**), and **DMA** if asynchronous API is used.
- Provide the configured bus instance to the sensor library.
- When necessary, configure the interrupt pin and implement platform-specific delay functions.

### 2.a Source code integration

Typically, the code can be used as presented in the example folder. However, to generalize the driver, a `BusOperation` trait is used. This allows for a generic bus that could be either I2C or SPI. The `util` folder wraps the trait in the [st-mems-bus](https://github.com/STMicroelectronics/st-mems-rust-drivers/tree/main/util/st-mems-bus) crate, enabling the same trait to be shared across all sensors and used simultaneously without redefining the trait. The configuration depends on the framework being used.

#### Selecting async or blocking API

By default you use the asynchronous API:

```rust
use lsm6dsv16x_rs::asynchronous as lsm6dsv16x;
use lsm6dsv16x::prelude::*;
```

To use the blocking API instead:
```rust
use lsm6dsv16x_rs::blocking as lsm6dsv16x;
use lsm6dsv16x::prelude::*;
```

Below is minimal example using `LSM6DSV16X` as sensor reference. Implementation for Embassy (Async) and STM32 frameworks are provided:

- **Embassy Async I2C**:
   ```rust
   use embassy_stm32::{bind_interrupts, Config};
   use embassy_stm32::exti::ExtiInput;
   use embassy_stm32::gpio::{Input, Pull};
   use embassy_stm32::i2c::{self, Config as I2cConfig, I2c};
   use embassy_stm32::time::khz;
   use embassy_stm32::peripherals::{self, USART2};
   use embassy_time::Delay;

   // Async import
   use lsm6dsv16x_rs::asynchronous as lsm6dsv16x;
   // if blocking feature is enabled
   // use lsm6dsv16x_rs::blocking as lsm6dsv16x;
   use lsm6dsv16x::prelude::*;

   bind_interrupts!(struct Irqs {
      USART2 => BufferedInterruptHandler<USART2>;
      I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
      I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
   });

   // main section
   #[embassy_executor::main]
   async fn main(_spawner: Spawner) {
      let p = embassy_stm32::init(Default::default());

      let i2c: I2c<_> = I2c::new(
         p.I2C1, // TBD: define the I2C channel as needed
         p.PB8, // TBD: define the scl route
         p.PB9, // TBD: define the sda route
         Irqs,
         p.DMA1_CH6, // TBD: provide tx Dma if available
         p.DMA1_CH0, // TBD: provide rx Dma if available
         khz(400),
         I2cConfig::default(),
      );

      let mut delay = Delay;

      let interrupt_pin = p.PC0; // TBD: define the interrupt pin accordingly
      let exti = p.EXTI0; // TBD: define the EXTI related to the interrupt pin
      let interrupt = Input::new(interrupt_pin, Pull::None);
      let mut interrupt = ExtiInput::new(interrupt, exti);

      let i2c_addr = lsm6dsv16x::I2CAddress::I2cAddH; // TBD: depends on whether SDA0 is high or not; see sensor's datasheet.

      let mut sensor = Lsm6dsv16x::new_i2c(i2c, i2c_addr, delay).unwrap();
   }
   ```

- **STM32 I2C (blocking)**:
   ```rust
   use stm32f4xx_hal::{
      i2c::{DutyCycle, I2c, Mode},
      pac,
      prelude::*,
      serial::{config::Config, Serial},
   };

   // Blocking import
   use lsm6dsv16x_rs::blocking as lsm6dsv16x;
   // If your project uses an async runtime on STM32, you can instead use:
   // use lsm6dsv16x_rs::asynchronous as lsm6dsv16x;
   use lsm6dsv16x::prelude::*;

   // main section
   #[entry]
   fn main() -> ! {
      let dp = pac::Peripherals::take().unwrap();
      let cp = cortex_m::Peripherals::take().unwrap();

      let rcc = dp.RCC.constrain();
      let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();

      let mut delay = cp.SYST.delay(&clocks);
      let tim1 = dp.TIM1.delay_us(&clocks);

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

      let i2c_addr = lsm6dsv16x::I2CAddress::I2cAddH; // TBD: depends on whether SDA0 is high or not; see sensor's datasheet.

      let mut sensor = Lsm6dsv16x::new_i2c(i2c, i2c_addr, tim1).unwrap();
   }
   ```

- **Embassy Async SPI**
   ```rust
   use core::cell::RefCell;
   use static_cell::StaticCell;
   use embassy_sync::blocking_mutex::NoopMutex;
   use embassy_time::{Delay, Duration, Timer, WithTimeout};
   use embassy_time::Delay;
   use embedded_hal_bus::spi::ExclusiveDevice as SpiDevice; // or the actual type used

   use embassy_stm32 as hal;
   use hal::gpio::{Level, Output, Speed};
   use hal::spi::{Spi, Config as SpiConfig};
   use hal::{bind_interrupts, peripherals};

   // Async import (default)
   use lsm6dsv16x_rs::asynchronous as lsm6dsv16x;
   // If blocking feature is enabled and you want blocking API instead:
   // use lsm6dsv16x_rs::blocking as lsm6dsv16x;
   use lsm6dsv16x::prelude::*;

   // Shared SPI bus (async)
   static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, peripherals::SPI1>>>> = StaticCell::new();

   // main section
   #[embassy_executor::main]
   async fn main(_spawner: Spawner) {
      let p = embassy_stm32::init(Default::default());

      let mut config = SpiConfig::default();
      config.mode = MODE_3; // TBD: define MODE
      config.frequency = embassy_stm32::time::Hertz(100_000); // TBD: define frequency

      // Async SPI constructor — replace DMA types if you want DMA-based transfers
      let spi = Spi::new(
         p.SPI1,                 // SPI peripheral
         p.PA1,                  // SCK - TBD: choose correct pin
         p.PA7,                  // MOSI - TBD
         p.PA6,                  // MISO - TBD
         config,
      );

      let mut delay = Delay;

      let bus = NoopMutex::new(RefCell::new(spi));
      let bus = SPI_BUS.init(bus);

      let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh); // TBD: define Chip select (CS) settings

      let spi_dev = SpiDevice::new(bus, cs);

      // For the async API this will typically be used inside an async context:
      // let mut sensor = lsm6dsv16x::Lsm6dsv16x::new_spi(spi_dev).await?;
      let mut sensor = Lsm6dsv16x::new_spi(spi_dev, delay);
   }
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

   // Blocking import
   use lsm6dsv16x_rs::blocking as lsm6dsv16x;
   // If your project uses an async runtime on STM32, you can instead use:
   // use lsm6dsv16x_rs::asynchronous as lsm6dsv16x;
   use lsm6dsv16x::prelude::*;

   // main section
   #[entry]
   fn main() -> ! {
      let dp = pac::Peripherals::take().unwrap();
      let cp = cortex_m::Peripherals::take().unwrap();

      let rcc = dp.RCC.constrain();
      let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();

      let mut delay = cp.SYST.delay(&clocks);
      let tim1 = dp.TIM1.delay_us(&clocks);

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

      let mut sensor = Lsm6dsv16x::new_spi(spi_dev, tim1);
   }
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
