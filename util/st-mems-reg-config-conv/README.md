# st-mems-reg-config-conv Library

The Registers Configuration Converter is designed to facilitate the conversion of JSON/UCF registers configuration files, generated using STMicroelectronics tools, into Rust code. This library streamlines the process of integrating device configurations into Rust projects, supporting no_std environment.

## Installation and usage
To use the Registers Configuration Converter in your project, follow these steps:

### Step 1: Add Dependency

Include the Registers Configuration Converter as a dependency in your Cargo.toml file:

```[Toml]
[dependencies]
st-mems-reg-config-conv = {path = "path_to_lib"}
```

### Step 2: Enable std Features for Build

For the build process std is required by the parser. But the library could still compile for no_std projects.

```[Toml]
[dependencies]
st-mems-reg-config-conv = {path = "path_to_lib", features = ['std']}
```

### Step 3: Build script Integration

In your build script, include the parser;

```[Rust]
use st_mems_reg_config_conv::parser;
```

### Step 4: Build Main Fucntion Implementation

Inside the build script, add this code in the main function to specify the input and output files alongside the name of the array that will contain the entries.

```[Rust]
let input_file = Path::new("path_to_reg_config");
let output_file = Path::new("src/rs_file_output");
parser::generate_rs_from_json(&input_file, &output_file, "JsonEntries");
```

## Usage in no_std Projects

The Registers Configuration Converter is designed to be used in no_std projects by default. However, the parsers require linking to the standard library, necessitating the library's inclusion as both a regular dependency and a build dependency. In a std environment, this dual import is not necessary.
