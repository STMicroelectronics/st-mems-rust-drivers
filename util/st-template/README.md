# {{project-name}}

A Rust embedded project for STM32 microcontrollers using {%- if framework == "stm32rs" %} STM32 HAL{%- else %} Embassy{%- endif %}.

## Hardware Target

- **MCU**: {{mcu}}
- **Framework**: {{framework}}

## Getting Started

### Prerequisites

1. Install Rust with the embedded target:
```bash
rustup target add thumbv7em-none-eabihf
```

2. Install probe-rs for flashing and debugging:
```bash
cargo install probe-rs-tools --locked
```

### Building

```bash
cargo build --release
```

### Flashing

Connect your ST-Link debugger and run:
```bash
cargo run --release
```

## Examples

This template includes several examples demonstrating common embedded peripherals:

### Available Examples

- **blink** - Basic LED blinking (PC13)
- **i2c** - I2C communication and device scanning (PB8/PB9)
- **uart** - UART communication (PA9/PA10)
- **pwm** - PWM output for LED brightness control (PA8)

### Running Examples

```bash
# Run the I2C example
cargo run --example i2c --release

{%- if sensor == "no_sensor" %}

{%- else %}
# Run WhoAmI example
cargo run --example whoAmI --release

{%- endif %}
```

### Example Pin Assignments

| Example | Pins Used | Description |
|---------|-----------|-------------|
| blink   | PC13      | Onboard LED (most STM32 boards) |
| i2c     | PB8 (SCL), PB9 (SDA) | I2C1 interface |
| uart    | PA9 (TX), PA10 (RX) | USART1 interface |
| pwm     | PA8       | TIM1_CH1 PWM output |

## Project Structure

{%- if framework == "stm32rs" %}
This project uses the STM32 HAL (Hardware Abstraction Layer) which provides:
- Direct register access with type safety
- Blocking APIs with explicit error handling
- Fine-grained control over peripherals
- Lower memory overhead

### Key Files

- `src/main.rs` - Main application code
- `memory.x` - Linker script defining memory layout
- `.cargo/config.toml` - Cargo configuration for embedded target

{%- else %}
This project uses Embassy, an async embedded framework that provides:
- Async/await support for embedded programming
- Built-in drivers for many peripherals
- Efficient task scheduling
- Modern Rust patterns

### Key Files

- `src/main.rs` - Main application code with async main
- `defmt.toml` - Logging configuration
- `.cargo/config.toml` - Cargo configuration for embedded target

### Logging

This project uses `defmt` for efficient logging. Logs will appear in your probe-rs terminal when running the application.

{%- endif %}

## Development

### Debugging

You can use probe-rs for debugging:
```bash
probe-rs debug --chip {{mcu}}
```

Or use GDB with your preferred setup.

### Memory Usage

Check memory usage with:
```bash
cargo size --release
```

## License

This project is licensed under your preferred license.
