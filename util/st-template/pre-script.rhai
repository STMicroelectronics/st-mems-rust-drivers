let chip = variable::get("mcu").to_lower();
let sensor = variable::get("sensor");
let sensor_capitalized = capitalize(sensor);

// split sensor parts (example based on: stm32l151r8t6)
let name = chip;
let family = name[0..7];               // "stm32l1"
let series = name[6..9];              // "151"
let flash = name[10];                  // "8"

print(series);

let svd_name = name[0..9];

// optional values
// let package_name = name[11];           // "t"
// let temp = name[12];                   // "6"

// compute flash and ram size
let flash_size_kb = switch flash {
    '4' => 16,
    '6' => 32,
    '8' => 64,
    'b' => 128,
    'c' => 256,
    'd' => 384,
    'e' => 512,
    'f' => 1024,
    _   => throw "Unknown flash size"
};

let ram_kb = get_ram_kb(family, series, flash_size_kb);

// Get TCM/CCM memory configurations
let tcm_config = get_tcm_config(family, series);
let itcm_kb = tcm_config.itcm;
let dtcm_kb = tcm_config.dtcm;
let ccm_kb = tcm_config.ccm;

// Determine target triple based on MCU family
let target = get_target(family);

// Determine OpenOCD target configuration
let openocd_target = get_openocd_target(family);

fn capitalize(s) {
    if s.len > 0 {
        s[0].to_upper() + s[1..s.len]
    } else {
        s
    }
}

/// Returns target triple for different STM32 families
fn get_target(family) {
    switch family {
        // Cortex-M0/M0+
        "stm32f0" | "stm32g0" | "stm32c0" | "stm32l0" => "thumbv6m-none-eabi",

        // Cortex-M3
        "stm32f1" | "stm32l1" => "thumbv7m-none-eabi",

        // Cortex-M4F (with FPU)
        "stm32f3" | "stm32f4" | "stm32g4" | "stm32l4" | "stm32wl" => "thumbv7em-none-eabihf",

        // Cortex-M7F (with FPU and double precision)
        "stm32f7" | "stm32h7" => "thumbv7em-none-eabihf",

        _ => "thumbv7em-none-eabihf" // default
    }
}

/// Returns OpenOCD target configuration file name (without .cfg extension)
fn get_openocd_target(family) {
    switch family {
        "stm32f0" => "stm32f0x",
        "stm32f1" => "stm32f1x",
        "stm32f2" => "stm32f2x",
        "stm32f3" => "stm32f3x",
        "stm32f4" => "stm32f4x",
        "stm32f7" => "stm32f7x",
        "stm32g0" => "stm32g0x",
        "stm32g4" => "stm32g4x",
        "stm32h7" => "stm32h7x",
        "stm32l0" => "stm32l0",
        "stm32l1" => "stm32l1",
        "stm32l4" => "stm32l4x",
        "stm32l5" => "stm32l5x",
        "stm32c0" => "stm32c0x",
        "stm32wl" => "stm32wlx",
        "stm32wb" => "stm32wbx",
        _ => "stm32f4x" // default fallback
    }
}

/// Returns TCM and CCM configuration for STM32 families
/// Returns a map with itcm, dtcm, and ccm sizes in KB (0 if not available)
fn get_tcm_config(family, series) {
    let config = #{
        itcm: 0,
        dtcm: 0,
        ccm: 0
    };

    switch family {
        // STM32F3 has CCM (Core Coupled Memory)
        "stm32f3" => {
            config.ccm = switch series {
                "301" | "318" => 0,
                "302" | "303" => 8,
                "328" | "334" => 4,
                "358" => 8,
                "373" | "378" => 0,
                "398" => 8,
                _ => 0
            };
        },

        // STM32F4 has CCM
        "stm32f4" => {
            config.ccm = switch series {
                "401" | "410" | "411" | "412" => 0,
                "405" | "407" | "415" | "417" => 64,
                "413" | "423" => 64,
                "427" | "429" | "437" | "439" => 64,
                "446" => 0,
                "469" | "479" => 64,
                _ => 0
            };
        },

        // STM32F7 has ITCM and DTCM
        "stm32f7" => {
            // F7 series has 16KB ITCM and 64-128KB DTCM
            config.itcm = 16;
            config.dtcm = switch series {
                "722" | "723" | "730" | "732" | "733" => 64,
                "745" | "746" | "750" | "756" => 64,
                "765" | "767" | "768" | "769" | "777" | "778" | "779" => 128,
                _ => 64
            };
        },

        // STM32H7 has ITCM and DTCM (not in your original script, but adding for completeness)
        "stm32h7" => {
            config.itcm = 64;
            config.dtcm = 128;
        },

        _ => {
            // No TCM/CCM for other families
        }
    };

    config
}

/// Returns RAM size in KB for a given STM32 family, series, and flash size.
/// Throws an error if the combination is unknown.
fn get_ram_kb(family, series, flash_size_kb) {
    switch family {
        // STM32F0 family
        "stm32f0" => switch series {
            "030" | "031" | "038" => switch flash_size_kb {
                16 | 32 => 4,
                64 => 8,
                256 => 32,
                _ => throw "Unknown RAM size for STM32F0[030/031/038]"
            },
            "042" | "048" => 6,
            "051" | "058" => 8,
            "070" | "071" | "072" | "078" => switch flash_size_kb {
                32 => 6,
                64 | 128 => 16,
                _ => throw "Unknown RAM size for STM32F0[070/071/072/078]"
            },
            "091" | "098" => 32,
            _ => throw "Unknown RAM size for STM32F0"
        },

        // STM32F1 family
        "stm32f1" => switch series {
            "100" => switch flash_size_kb {
                16 | 32 => 4,
                64 | 128 => 8,
                256 => 24,
                384 | 512 => 32,
                _ => throw "Unknown RAM size for STM32F100"
            },
            "101" | "102" => switch flash_size_kb {
                16 => 4,
                32 => 6,
                64 => 10,
                128 => 16,
                256 => 32,
                384 | 512 => 48,
                768 | 1024 => 80,
                _ => throw "Unknown RAM size for STM32F101/102"
            },
            "103" => switch flash_size_kb {
                16 => 6,
                32 => 10,
                64 | 128 => 20,
                256 => 48,
                384 | 512 => 64,
                768 | 1024 => 96,
                _ => throw "Unknown RAM size for STM32F103"
            },
            "105" | "107" => 64,
            _ => throw "Unknown RAM size for STM32F1"
        },

        // STM32F3 family
        "stm32f3" => switch series {
            "301" | "318" => 16,
            "302" => switch flash_size_kb {
                32 | 64 => 16,
                128 => 32,
                256 => 40,
                384 | 512 => 64,
                _ => throw "Unknown RAM size for STM32F302"
            },
            "303" => switch flash_size_kb {
                32 | 64 => 12,
                128 => 32,
                256 => 40,
                384 | 512 => 64,
                _ => throw "Unknown RAM size for STM32F303"
            },
            "328" | "334" => 12,
            "358" => 40,
            "373" => switch flash_size_kb {
                64 => 16,
                128 => 24,
                256 => 32,
                _ => throw "Unknown RAM size for STM32F373"
            },
            "378" => 32,
            "398" => 64,
            _ => throw "Unknown RAM size for STM32F3"
        },

        // STM32F4 family
        "stm32f4" => switch series {
            "401" => switch flash_size_kb {
                128 | 256 => 64,
                384 | 512 => 96,
                _ => throw "Unknown RAM size for STM32F401"
            },
            "405" | "407" | "415" | "417" => 128, // 112+16
            "410" => 32,
            "411" => 128,
            "412" => 256,
            "413" | "423" => 320, // 256+64
            "427" | "429" | "437" | "439" => 192, // 112+16+64
            "446" => 128, // 112+16
            "469" | "479" => 320, // 160+32+128
            _ => throw "Unknown RAM size for STM32F4"
        },

        // STM32F7 family
        "stm32f7" => switch series {
            "722" | "723" | "730" | "732" | "733" => 192, // 176+16
            "745" | "746" | "750" | "756" => 256, // 240+16
            "765" | "767" | "768" | "769" | "777" | "778" | "779" => 384, // 368+16
            _ => throw "Unknown RAM size for STM32F7"
        },

        // STM32G0 family
        "stm32g0" => switch series {
            "030" | "031" | "041" => 8,
            "050" | "051" | "061" => 16,
            "070" | "071" | "081" => 32,
            "0b0" | "0b1" | "0c1" => 128,
            _ => throw "Unknown RAM size for STM32G0"
        },

        // STM32G4 family
        "stm32g4" => switch series {
            "431" | "441" => 32, // 16+6+10
            "473" | "474" | "483" | "484" => 128, // 80+16+32
            "491" | "4a1" => 112, // 80+16+16
            _ => throw "Unknown RAM size for STM32G4"
        },

        // STM32C0 family
        "stm32c0" => switch series {
            "011" => 6,
            "031" => 12,
            _ => throw "Unknown RAM size for STM32C0"
        },

        // STM32L0 family
        "stm32l0" => switch series {
            "010" | "011" | "021" | "031" | "041" | "051" | "052" | "053" | "062" | "063" => switch flash_size_kb {
                8 | 16 => 2,
                32 | 64 => 8,
                128 => 20,
                _ => throw "Unknown RAM size for STM32L0"
            },
            "071" | "072" | "073" | "081" | "082" | "083" => 20,
            _ => throw "Unknown RAM size for STM32L0"
        },

        // STM32L1 family
        "stm32l1" => switch series {
            "100" => switch flash_size_kb {
                32 => 4,
                64 => 8,
                128 => 10,
                256 => 16,
                _ => throw "Unknown RAM size for STM32L100"
            },
            "151" | "152" => switch flash_size_kb {
                32 | 64 => 10,
                128 => 16,
                256 => 32,
                384 => 48,
                512 => 80,
                _ => throw "Unknown RAM size for STM32L151/152"
            },
            "162" => switch flash_size_kb {
                256 => 32,
                384 => 48,
                512 => 80,
                _ => throw "Unknown RAM size for STM32L162"
            },
            _ => throw "Unknown RAM size for STM32L1"
        },

        // STM32L4 family
        "stm32l4" => switch series {
            "412" | "422" => 40, // 32+8
            "431" | "432" | "433" | "442" | "443" => 64, // 48+16
            "451" | "452" | "462" => 160,
            "471" | "475" | "476" | "486" => 128,
            "496" | "4a6" => 320, // 256+64
            _ => throw "Unknown RAM size for STM32L4"
        },

        // STM32WL family
        "stm32wl" => switch series {
            "54" | "55" => 64, // 32+32
            "e4" | "e5" => switch flash_size_kb {
                64 => 20,
                128 => 48,
                256 => 64,
                _ => throw "Unknown RAM size for STM32WL"
            },
            _ => throw "Unknown RAM size for STM32WL"
        },

        _ => throw "Unknown STM32 family"
    }
}


/// Configuration of the sensor who_am_i example
let address = "0x00";
let delay = ", delay";
let who_am_i_fn = "device_id_get()";
let id = "ID";

switch sensor {
    "iis2dlpc" => {
        address = "I2CAddress::I2cAddH";
    },
    "lps22hh" => {
        address = "I2CAddress::AddressH";
        delay = "";
    },
    "lsm6dso16is" | "ism330is" => {
        address = "I2CAddress::I2cAddL";
    },
    "lsm6dsv16x" => {
        address = "I2CAddress::I2cAddH";
    },
    "lsm6dsv320x" | "lsm6dsv80x" | "ism6hg256x" => {
        address = "I2CAddress::I2cAddL";
    },
    "iis2mdc" => {
        address = "I2CAddress::I2cAdd";
        id = "IIS2MDC";
    },
    "ilps22qs" => {
        address = "I2CAddress::I2cAdd";
        who_am_i_fn = "id_get()";
        id = "ILPS22QS_ID";
    },
    "ism330dhcx" => {
        address = "I2CAddress::I2cAddH";
        id = "ISM330DHCX_ID";
    },
    "lis2dux12" | "lis2duxs12" | "iis2dulpx" => {
        address = "I2CAddress::I2cAddH";
    },
    "lis2mdl" => {
        address = "I2CAddress::I2cAdd";
        delay = "";
    },
    "lps22df" => {
        address = "I2CAddress::I2cAddH";
    }
}

// Set all variables
variable::set("id", id);
variable::set("delay", delay);
variable::set("who_am_i_fn", who_am_i_fn);
variable::set("address", address);
variable::set("sensor_capitalized", sensor_capitalized);
variable::set("ram_kb", ram_kb.to_string());
variable::set("flash_size_kb", flash_size_kb.to_string());
variable::set("target", target);
variable::set("openocd_target", openocd_target);
variable::set("itcm_kb", itcm_kb.to_string());
variable::set("dtcm_kb", dtcm_kb.to_string());
variable::set("ccm_kb", ccm_kb.to_string());
variable::set("svd_name", svd_name);
