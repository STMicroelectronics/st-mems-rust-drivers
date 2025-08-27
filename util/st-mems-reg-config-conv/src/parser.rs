use std::fs;
use std::println;
use std::path::Path;
use std::{vec, format};
use std::string::ToString;
use std::vec::Vec;
use std::string::String;
use serde::Deserialize;
use crate::ucf_entry::{MemsUcfOp, UcfLineExt};
use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Deserialize)]
#[allow(dead_code)]
struct JsonFormat {
    #[serde(rename = "type")]
    json_type: String,
    version: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Application {
    name: String,
    version: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
#[allow(dead_code)]
enum ConfigurationEntry {
    Comment { comment: String },
    Operation { 
        #[serde(rename = "type")]
        config_type: ConfigType, 
        address: Option<String>, 
        data: String 
    },
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(dead_code)]
enum ConfigType {
    #[serde(rename = "write")]
    Write,
    #[serde(rename = "delay")]
    Delay,
    // Add other types as needed
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Sensor {
    name: Vec<String>,
    configuration: Vec<ConfigurationEntry>,
    outputs: Vec<Output>
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Output {
    name: String,
    core: String,
    #[serde(rename = "type")]
    output_type: String,
    len: String,
    reg_addr: String,
    reg_name: String,
    // results
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct JsonData {
    json_format: JsonFormat,
    application: Application,
    description: String,
    sensors: Vec<Sensor>,
}

#[repr(C)]
pub enum FileType {
    Json,
    Ucf
}

/// Handles the .rs file generation from UCF/JSON configurations files for MLC/FSM/ISPU.
///
/// Rust code could use the more specific alternatives: generate_rs_from_json,
/// generate_rs_from_ucf; the purpose of this function is to expose an interface for C programs
///
/// # Arguments
///
/// * `input_file`: c_char string pointing to the path of the input file (.ucf/.json)
/// * `output_file`: c_char string pointing to the path where to save the .rs file generated
/// * `array_name`: c_char string used to set the array's name containing the configurations
/// * `sensor_id`: c_char string used to ensure load of right configuration (applied only for json
///   configurations)
/// * `file_type`: FileType enum is used to select how to handle input_file (allowed values are
///   Json/Ucf).
///
/// # Safety
/// - Each c_char string should contain a valid nul terminator at the end of the string
/// - Each c_char is a valid pointer:
///     - entire memory range is contained in a single allocation
///     - should be non-null even for a zero-length content
/// - Memory pointed by c_char pointers should not change during the execution of the function
/// - The nul terminator for c_char pointers mut be before isize::MAX
#[unsafe(no_mangle)]
pub unsafe extern "C" fn generate_rs(
    input_file: *const c_char,
    output_file: *const c_char,
    array_name: *const c_char,
    sensor_id: *const c_char,
    file_type: FileType,
) -> i32 {

    // Check for null pointers
    if input_file.is_null() || output_file.is_null() || array_name.is_null() || sensor_id.is_null() {
        return -1; // error code
    }

    let input_file = unsafe { CStr::from_ptr(input_file) }.to_str().unwrap();
    let output_file = unsafe { CStr::from_ptr(output_file) }.to_str().unwrap();
    let array_name = unsafe { CStr::from_ptr(array_name) }.to_str().unwrap();
    let sensor_id = unsafe { CStr::from_ptr(sensor_id) }.to_str().unwrap();

    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);

    match file_type {
        FileType::Json => generate_rs_from_json(input_path, output_path, array_name, sensor_id, false),
        FileType::Ucf => generate_rs_from_ucf(input_path, output_path, array_name),
    };

    0
}

pub fn generate_rs_from_json(input_file: &Path, output_file: &Path, array_name: &str, sensor_id: &str, verbose: bool) {
    let content = fs::read_to_string(input_file).expect("Failed to read input file");
    let json_data: JsonData = serde_json::from_str(&content).expect("Failed to parse JSON");

    // Parse configuration lines
    let mut config_lines = vec![];
    let mut found_one_config = false;
    for sensor in json_data.sensors {
        for name in &sensor.name {
            println!("{name}");
        }
        if !sensor.name.contains(&sensor_id.to_uppercase()) {
            continue;
        }
        for config in sensor.configuration {
            match config {
                ConfigurationEntry::Comment { .. } => {
                    
                },
                ConfigurationEntry::Operation {
                    config_type, address, data
                } => {
                    if config_type == ConfigType::Write {
                        let address = address.unwrap();
                        let address = u8::from_str_radix(&address[2..], 16).expect("Invalid address format");
                        let data = u8::from_str_radix(&data[2..], 16).expect("Invalid data format");
                        config_lines.push(UcfLineExt {
                            op: MemsUcfOp::Write,
                            address,
                            data,
                        });
                    } else if config_type == ConfigType::Delay {
                        let data = data.parse::<u8>().expect("Cannot parse milliseconds of delay");
                        config_lines.push(UcfLineExt {
                            op: MemsUcfOp::Delay,
                            address: 0x0,
                            data
                        });
                    }
                }
            }
        }

        if verbose {
            println!("-- Summary of outputs generated by json configuration, open json for more details --");

            for output in sensor.outputs {
                println!("{}, {}", output.name, output.reg_name);
            }
        }

        found_one_config = true;
        break;
    }

    if !found_one_config {
        panic!("No configuration found for the sensor id");
    }

   

    // Generate Rust code
    let mut output = String::new();
    output.push_str(r#"// DO NOT EDIT. File autogenerated during each build from "#);
    output.push_str(&format!("JSON file: {} \n", input_file.display()).to_string());
    output.push_str(r#"// Change build.rs script to change JSON source file"#);
    output.push_str("\nuse st_mems_reg_config_conv::ucf_entry::*;\n\n");
    output.push_str(&format!("#[rustfmt::skip]\npub const {array_name}: [UcfLineExt; ").to_string());
    output.push_str(&config_lines.len().to_string());
    output.push_str("] = [\n");

    for config_line in &config_lines {
        let line = format!(
            "   UcfLineExt {{ op: MemsUcfOp::{}, address: 0x{:02X}, data: 0x{:02X} }},\n",
            config_line.op.to_string(),
            config_line.address,
            config_line.data
        );
        output.push_str(&line);
    }

    output.push_str("];\n");

    // Write to output file
    fs::write(output_file, output).expect("Failed to write output file");
}

pub fn generate_rs_from_ucf(input_file: &Path, output_file: &Path, array_name: &str) {
    let content = fs::read_to_string(input_file).expect("Failed to read input file");
    let mut lines = content.lines();

    // Skip comment lines
    for line in lines.by_ref() {
        if !line.starts_with("--") {
            break;
        }
    }

    // Parse remaining lines
    let mut ucf_lines = vec![]; 
    for line in lines {
        if let Some(ucf_entry) = parse_line(line) {
            ucf_lines.push(ucf_entry);
        }
    }

    // Generate Rust code
    let mut output = String::new();
    output.push_str(r#"// DO NOT EDIT. File autogenerated during each build from "#);
    output.push_str(&format!("ucf file: {} \n", input_file.display()).to_string());
    output.push_str(r#"// Change build.rs script to change ucf source file"#);
    output.push_str("\nuse st_mems_reg_config_conv::ucf_entry::*;\n\n");
    output.push_str(&format!("#[rustfmt::skip]\npub const {array_name}: [UcfLineExt; ").to_string());
    output.push_str(&ucf_lines.len().to_string());
    output.push_str("] = [\n");

    for ucf_line in &ucf_lines {
        let line = match ucf_line.op {
            MemsUcfOp::Delay => &format!(
                        "   UcfLineExt {{ op: MemsUcfOp::{}, address: 0x{:02X}, data: {} }},\n",
                        ucf_line.op.to_string(), ucf_line.address, ucf_line.data
                ),
            _ => &format!(
                        "   UcfLineExt {{ op: MemsUcfOp::{}, address: 0x{:02X}, data: 0x{:02X} }},\n",
                        ucf_line.op.to_string(), ucf_line.address, ucf_line.data
                )
        };
        output.push_str(line);
    }

    output.push_str("];\n");

    // Write to output file
    fs::write(output_file, output).expect("Failed to write output file");
}

fn parse_line(line: &str) -> Option<UcfLineExt> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() == 2 && parts[0] == "WAIT" {
        return Some(UcfLineExt {
            op: MemsUcfOp::Delay,
            address: 0,
            data: parts[1].parse::<u8>().expect("Cannot parse milliseconds of delay")
        })
    }
    if parts.len() == 3 && parts[0] == "Ac" {
        let address = u8::from_str_radix(parts[1], 16).ok()?;
        let data = u8::from_str_radix(parts[2], 16).ok()?;
        return Some(UcfLineExt {
            op: MemsUcfOp::Write,
            address,
            data
        })
    } 

    None
}
