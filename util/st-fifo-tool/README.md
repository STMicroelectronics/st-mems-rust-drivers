# st-fifo-tool
[![Crates.io][crates-badge]][crates-url]
[![BSD 3-Clause licensed][bsd-badge]][bsd-url]

[crates-badge]: https://img.shields.io/crates/v/st-fifo-tool
[crates-url]: https://crates.io/crates/st-fifo-tool
[bsd-badge]: https://img.shields.io/crates/l/st-fifo-tool
[bsd-url]: https://opensource.org/licenses/BSD-3-Clause

This repository contains a set of utilities useful to interface with the ST MEMS TAG-based IMUs sensor FIFO: it provides the capability to decode and decompress the data samples.

## Api Example

The APIs are the following:

```rust
// Create a st_fifo_tool config:
let config = st_fifo_tool::Config {
    device: st_fifo_tool::DeviceType::...,
    bdr_xl: ...,
    bdr_gy: ...,
    bdr_vsens: ...
};

// Initialize the st_fifo_tool
let mut fifo = st_fifo_tool::FifoData::init(&config).unwrap();

// Initialize structs to hold the input/output data
let SLOT_NUMBER: u8 = ...;
let mut raw_slot = [st_fifo_tool::RawSlot::default(); SLOT_NUMBER as usize];
let mut out_slot = [st_fifo_tool::OutSlot::default(); SLOT_NUMBER as usize];

// Reads data (inside a loop)
let mut out_slot_size: u16 = 0;
let f_data = sensor.fifo_out_raw_get().unwrap(); // get raw data from some sensor
let fifo_status = sensor.fifo_status_get().unwrap();
let slots = fifo_status.fifo_level;

for i in 0..slots {
    raw_slot[i as usize].fifo_data_out[0] = ((f_data.tag as u8) << 3) | (f_data.cnt << 1);
    raw_slot[slots as usize].fifo_data_out[1..].copy_from_slice(&f_data.data);
}

// Decode and sort the data
fifo.decode(&mut out_slot, &raw_slot, &mut out_slot_size, slots);
fifo.sort(&mut out_slot, out_slot_size);

// Count how many samples for SensorType 
let mut solts_for_acc = 0 // accelerometer or gyroscope
fifo.extract_sensor(&mut slots_for_acc, &out_slot, out_slot_size, st_fifo_tool::SensorType::Accelerometer);

// Extract samples from the SensorType
fifo.extract_sensor(&mut acc_slot, &out_slot, out_slot_size, st_fifo_tool::SensorType::Accelerometer);

```

## Repository overview

This utility is structured as follows:  

- [lib.rs](./src/lib.rs): Provides the *FifoTool* strucing to decode/sort/extract raw fifo data. *RawSlot* and *OutSlot* are auxiliary structures used as input/output data.
    - RawSlot: Used to read the input data to be processed (the example provides full details). 
    - OutSlot: Contains the output generated after decoding: a timestamp, a sensor tag useful to interpret the SensorData.
- [sensor_data.rs](./src/sensor_data.rs): Defines the *SensorData* struct, it holds the raw data and provide methods to convert to various output depending on the tag value.

------

**More information: [http://www.st.com](http://st.com/MEMS)**

**Copyright © 2025 STMicroelectronics**

