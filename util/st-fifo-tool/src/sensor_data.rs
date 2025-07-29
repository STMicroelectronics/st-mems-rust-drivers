#[derive(Clone, Copy, Default)]
pub struct SensorData {
    pub data: [i16; 3],
}

impl SensorData {
    pub fn from_u8_arr(&mut self, data: &[u8]) {
        for i in 0..3 {
            self.data[i] = (data[i * 2] as i16) << 8 | data[i * 2 + 1] as i16;
        }
    }

    pub fn to_axis(&self) -> AxisData {
        AxisData {
            x: self.data[0],
            y: self.data[1],
            z: self.data[2],
        }
    }

    pub fn to_temperature(&self) -> TemperatureData {
        TemperatureData {
            temp: self.data[0]
        }
    }

    pub fn to_step_counter(&self) -> StepCounterData {
        StepCounterData {
            steps: self.data[0] as u16,
            steps_t: [
                (self.data[1] & 0xFF) as u8,
                (self.data[1] >> 8) as u8,
                (self.data[2] & 0xFF) as u8,
                (self.data[2] >> 8) as u8,
            ]
        }
    }

    pub fn to_quaternion(&self) -> QuaternionData {
        QuaternionData {
            qx: self.data[0] as u16,
            qy: self.data[1] as u16,
            qz: self.data[2] as u16,
        }
    }

    pub fn to_ext_sensor_nack(&self) -> ExtSensorNackData {
        ExtSensorNackData {
            nack: self.data[0] as u8,
        }
    }

    pub fn to_mlc_result(&self) -> MlcResultData {
        MlcResultData {
            mlc_res: (self.data[0] * 0xFF) as u8,
            mlc_idx: (self.data[0] >> 8) as u8,
            mlc_t: [
                (self.data[1] & 0xFF) as u8,
                (self.data[1] >> 8) as u8,
                (self.data[2] & 0xFF) as u8,
                (self.data[2] >> 8) as u8,
            ],
        }
    }

    pub fn to_mlc_filter_feature(&self) -> MlcFilterFeatureData {
        MlcFilterFeatureData {
            mlc_value: self.data[0] as u16,
            mlc_id: self.data[1] as u16,
            reserved: self.data[2] as u16,
        }
    }
}


#[derive(Clone, Copy, Default)]
pub struct AxisData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Clone, Copy, Default)]
pub struct TemperatureData {
    pub temp: i16,
}

#[derive(Clone, Copy, Default)]
pub struct StepCounterData {
    pub steps: u16,
    pub steps_t: [u8; 4],
}

#[derive(Clone, Copy, Default)]
pub struct QuaternionData {
    pub qx: u16,
    pub qy: u16,
    pub qz: u16,
}

#[derive(Clone, Copy, Default)]
pub struct ExtSensorNackData {
    pub nack: u8,
}

#[derive(Clone, Copy, Default)]
pub struct MlcResultData {
    pub mlc_res: u8,
    pub mlc_idx: u8,
    pub mlc_t: [u8; 4],
}

#[derive(Clone, Copy, Default)]
pub struct MlcFilterFeatureData {
    pub mlc_value: u16,
    pub mlc_id: u16,
    pub reserved: u16,
}

