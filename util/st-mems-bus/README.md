# st-mems-bus
[![Crates.io][crates-badge]][crates-url]
[![BSD 3-Clause licensed][bsd-badge]][bsd-url]

[crates-badge]: https://img.shields.io/crates/v/st-mems-bus
[crates-url]: https://crates.io/crates/st-mems-bus
[bsd-badge]: https://img.shields.io/crates/l/st-mems-bus
[bsd-url]: https://opensource.org/licenses/BSD-3-Clause

The st-mems-bus Library provides a unified and consistent API for accessing different types of communication buses. Currently, it supports both `SPI` and `I2C` buses, offering various modes for managing bus ownership and access.

Asynchronous support is offered through the [maybe-async](https://crates.io/crates/maybe-async) crate, which allows sharing a single implementation while generating code for blocking or asynchronous usage. End users can select synchronous mode by enabling the `is_sync` feature flag (by default, this flag is not set, providing async behavior).

## Traits

The crate offers several traits used by [st-mems-rust-drivers](https://github.com/STMicroelectronics/st-mems-rust-drivers/tree/main) drivers:
- **BusOperation**: The main trait shared among drivers. It provides basic read/write operations, allowing drivers to use a generic `BusOperation` regardless of whether the underlying bus is SPI or I2C.
- **SensorOperation**: A higher-level (and simpler) representation of `BusOperation`, which may replace `BusOperation` in the future.
- **MemBankFunctions**: Used to generalize drivers that split registers across multiple memory banks. When a memory bank is selected, others are not accessible. Memory protection mechanisms are implemented via [st-mem-bank-macro](https://crates.io/crates/st-mem-bank-macro).
- **EmbAdvFunctions**: Handles Embedded Advanced functions, which are special memory banks requiring specific read/write operations through a dedicated API.
- **RegisterOperation**: Implemented on top of each register struct to enable `read`, `write`, and `read_more` functionality.

## Usage

Add the library to your dependencies in `Cargo.toml`:

```toml
[dependencies]
st-mems-bus = "3.0.0"
```

## Features 
To keep the library lightweight, you can enable support for each bus type individually. By default, all bus types are included. Available features:

- **spi** - Enable support for SPI bus.
- **i2c** - Enable support for I2C bus.
- **is_sync** - Switches from asynchronous to synchronous blocking mode.

------

**More information: [http://www.st.com](http://st.com/MEMS)**

**Copyright © 2025 STMicroelectronics**
