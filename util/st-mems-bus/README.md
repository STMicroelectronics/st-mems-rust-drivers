# st-mems-bus Library

The st-mems-bus Library provides a unified and consistent API for accessing different types of communication buses. Currently, it supports both `SPI` and `I2C` buses, offering various modes for managing bus ownership and access.

## Access Modes

- **shared**:  
  This mode uses `RefCell` internally and calls `borrow_mut()` to ensure exclusive mutable access to the bus at runtime. While this introduces some overhead, it provides a simple mechanism to safely share the bus. More advanced sharing techniques are left to the user to implement as needed.

## Usage

Add the library to your dependencies in `Cargo.toml`:

```toml
[dependencies]
st-mems-bus = { path = "path_to_bus" }
```

## Features 
To keep the library lightweight, you can enable support for each bus type individually. By default, all bus types are included. Available features:

- **spi** - Enable support for SPI bus.
- **i2c** - Enable support for I2C bus.
