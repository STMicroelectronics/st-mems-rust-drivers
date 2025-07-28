# 1 - Introduction

This repository contains examples of *low-level* platform-independent drivers for [STMicroelectronics](https://www.st.com/mems) sensors. Sensor drivers and examples are written in Rust programming language.

The STMicroelectronics naming convention for driver repositories is:
 - `PARTNUMBER-pid-rs` (*e.g. lsm6dsv16x-pid-rs*) for *low-level platform-independent drivers (PID)*

### 1.a - Repository structure

This repository is structed with a folder for each sensor driver, named `xxxxxxx-pid-rs`, where `xxxxxxx` is the sensor part number.

Another folder, named  `util`, does not follow the above naming convention. It contains *other useful resources* such as libraries and crates. To `clone` the complete content of this folder, use the following command:

```git
git clone --recursive ***link***
```

### 1.b - Sensor driver folder structure

Every *sensor driver* folder contains the following:

- `xxxxxxx-pid-rs` : This folder is hosted as a submodule repository and published as a standalone crate on the [crates.io](https://crates.io/). Documentation can be found on the corresponding [crates.io](https://crates.io/) page or generated locally using the command: `cargo doc`.
- `xxxxxxx-pid-rs/examples`: This folder contains self-contained example projects to test the sensor. It may be necessary to modify the pin configuration or the I2C/SPI address as needed. The folder name of each examples includes the board used to test the sensor.
- `xxxxxxx-pid-rs/README`: additional info about the specific driver.

------

# 2 - Integration details
The driver is platform-independent. You need to set up the sensor hardware bus (ie. SPI or I2C), provide the bus to the sensor's library instance, and, when required: setup the interrupt pin and platform dependent delay.

### 2.a Source code integration

Typically, the code can be used as presented in the example folder. However, to generalize the driver, a `BusOperation` trait is used. This allows for a generic bus that could be either I2C or SPI. The `util` folder wraps the trait in the [st-mems-bus](https://github.com/STMicroelectronics/st-mems-rust-drivers/tree/main/util/st-mems-bus) crate, enabling the same trait to be shared across all sensors and used simultaneously without redefining the trait. The configuration depends on the framework being used. Below is a minimal example with `sensorDriverCrate` referring to the specific driver crate and `SensorDriver` referring to the library's struct. Implementation for Embassy and STM32 frameworks are provided:

- **Embassy**:
   ```rust
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

   let i2c_addr = sensorDriverCrate::I2CAddress::I2cAddH; // TBD: depends on whether SDA0 is high or not; see sensor details.

   let mut sensor = sensorDriverCrate::SensorDriver::new_i2c(i2c, i2c_addr).unwrap();
   ```

- **STM32**:
   ```rust
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

   let i2c_addr = sensorDriverCrate::I2CAddress::I2cAddH; // TBD: depends on whether SDA0 is high or not; see sensor details.

   let mut sensor = sensorDriverCrate::SensorDriver::new_i2c(i2c, i2c_addr).unwrap();

   ```


### 2.b Required properties

> * A rust compiler with a toolchain targeting the MCU

------

# 3 - Running examples

Examples are written for [STM32 Microcontrollers](https://www.st.com/en/microcontrollers.html) using the [NUCLEO_F401RE](https://github.com/STMicroelectronics/STMems_Standard_C_drivers/tree/master/_prj_NucleoF401) as primary platform. However, they can also serve as a guideline for every other platforms.

### 3.a Using STMicroelectronics evaluation boards

When using supported STMicroelectronics evaluation boards, the schematics provide information about which pins to use to setup the I2C or SPI communication with the sensor.

------

**More information: [http://www.st.com](http://st.com/MEMS)**

**Copyright (C) 2025 STMicroelectronics**

