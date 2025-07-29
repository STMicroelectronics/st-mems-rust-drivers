#![no_std]

mod sensor_data;

use core::fmt;
use sensor_data::*;

pub static DEVICES: [Device; 2] = [
    Device {
        bdr_acc: [0.0, 13.0, 26.0, 52.0, 104.0, 208.0, 416.0, 833.0, 1666.0, 3333.0, 6666.0, 1.625, 0.0, 0.0, 0.0, 0.0],
        bdr_gyr: [0.0, 13.0, 26.0, 52.0, 104.0, 208.0, 416.0, 833.0, 1666.0, 3333.0, 6666.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        bdr_vsens: [0.0, 13.0, 26.0, 52.0, 104.0, 208.0, 416.0, 0.0, 0.0, 0.0, 0.0, 1.625, 0.0, 0.0, 0.0, 0.0],
        dtime: [0, 3072, 1536, 768, 384, 192, 96, 48, 24, 12, 6, 24576, 0, 0, 0, 0],
        tag_valid_limit: 0x19,
    },
    Device {
        bdr_acc: [0.0, 1.875, 7.5, 15.0, 30.0, 60.0, 120.0, 240.0, 480.0, 960.0, 1920.0, 3840.0, 7680.0, 0.0, 0.0, 0.0],
        bdr_gyr: [0.0, 1.875, 7.5, 15.0, 30.0, 60.0, 120.0, 240.0, 480.0, 960.0, 1920.0, 3840.0, 7680.0, 0.0, 0.0, 0.0],
        bdr_vsens: [0.0, 1.875, 7.5, 15.0, 30.0, 60.0, 120.0, 240.0, 480.0, 960.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        dtime: [0, 24576, 6144, 3072, 1536, 768, 384, 192, 96, 48, 24, 12, 6, 0, 0, 0],
        tag_valid_limit: 0x1E,
    },
];

pub fn max<F: PartialOrd>(a: F, b: F) -> F {
    if a > b { a } else { b }
}

pub fn min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

#[derive(Clone)]
pub struct FifoData {
    fifo_ver: u8,
    tag_counter_old: u8,
    dtime_xl: u32,
    dtime_gy: u32,
    dtime_min: u32,
    dtime_xl_old: u32,
    dtime_gy_old: u32,
    timestamp: u32,
    last_timestamp_xl: u32,
    last_timestamp_gy: u32,
    bdr_chg_xl_flag: u8,
    bdr_chg_gy_flag: u8,
    last_data_xl: [i16; 3],
    last_data_gy: [i16; 3],
}

impl FifoData {

    pub fn init(conf: &Config) -> Result<Self, Status> {
        let bdr_xl = conf.bdr_xl;
        let bdr_gy = conf.bdr_gy;
        let bdr_vsens = conf.bdr_vsens;
        let bdr_max = max(bdr_xl, bdr_gy);
        let bdr_max = max(bdr_max, bdr_vsens);

        if bdr_xl < 0.0 || bdr_gy < 0.0 || bdr_vsens < 0.0 {
            return Err(Status::Err);
        }

        let fifo_ver = if conf.device < DeviceType::Lsm6dsv { 0 } else { 1 }; 

        let sensor_data = FifoData {
            fifo_ver,
            tag_counter_old: 0,
            dtime_xl: DEVICES[fifo_ver as usize].dtime[FifoData::bdr_get_index(&DEVICES[fifo_ver as usize].bdr_acc, bdr_xl)],
            dtime_gy: DEVICES[fifo_ver as usize].dtime[FifoData::bdr_get_index(&DEVICES[fifo_ver as usize].bdr_gyr, bdr_gy)],
            dtime_min: DEVICES[fifo_ver as usize].dtime[FifoData::bdr_get_index(&DEVICES[fifo_ver as usize].bdr_acc, bdr_max)],
            dtime_xl_old: DEVICES[fifo_ver as usize].dtime[FifoData::bdr_get_index(&DEVICES[fifo_ver as usize].bdr_acc, bdr_xl)],
            dtime_gy_old: DEVICES[fifo_ver as usize].dtime[FifoData::bdr_get_index(&DEVICES[fifo_ver as usize].bdr_gyr, bdr_gy)],
            timestamp: 0,
            last_timestamp_xl: 0,
            last_timestamp_gy: 0,
            bdr_chg_xl_flag: 0,
            bdr_chg_gy_flag: 0,
            last_data_xl: [0; 3],
            last_data_gy: [0; 3],
        };

        Ok(sensor_data)
    }

    pub fn decode(
        &mut self,
        fifo_out_slot: &mut [OutSlot],
        fifo_raw_slot: &[RawSlot],
        out_slot_size: &mut u16,
        stream_size: u16,
    ) -> Status {
        let mut j = 0;

        for i in 0..stream_size as usize {
            let tag = (fifo_raw_slot[i].fifo_data_out[0] & TagMask::Sensor as u8) >> TagShift::Sensor as u8;
            let tag_counter = (fifo_raw_slot[i].fifo_data_out[0] & TagMask::Counter as u8) >> TagShift::Counter as u8;

            if self.fifo_ver == 0 && FifoData::has_even_parity(fifo_raw_slot[i].fifo_data_out[0]) {
                return Status::Err;
            }

            if !self.is_tag_valid(tag) {
                return Status::Err;
            }

            if tag_counter != self.tag_counter_old && self.dtime_min != 0 {
                let diff_tag_counter = if tag_counter < self.tag_counter_old {
                    tag_counter + 4 - self.tag_counter_old
                } else {
                    tag_counter - self.tag_counter_old
                };

                self.timestamp += self.dtime_min * diff_tag_counter as u32;
            }

            let tag = Tag::try_from(tag).map_err(|_| Status::Err).unwrap();

            if tag == Tag::Odrchg {
                let bdr_acc_cfg = (fifo_raw_slot[i].fifo_data_out[6] & BdrMask::Xl as u8) >> BdrShift::Xl as u8;
                let bdr_gyr_cfg = (fifo_raw_slot[i].fifo_data_out[6] & BdrMask::Gy as u8) >> BdrShift::Gy as u8;
                let bdr_vsens_cfg = (fifo_raw_slot[i].fifo_data_out[4] & BdrMask::Vsens as u8) >> BdrShift::Vsens as u8;

                let bdr_xl = DEVICES[self.fifo_ver as usize].bdr_acc[bdr_acc_cfg as usize];
                let bdr_gy = DEVICES[self.fifo_ver as usize].bdr_gyr[bdr_gyr_cfg as usize];
                let bdr_vsens = DEVICES[self.fifo_ver as usize].bdr_vsens[bdr_vsens_cfg as usize];
                let bdr_max = max(max(bdr_xl, bdr_gy), bdr_vsens);

                self.dtime_xl_old = self.dtime_xl;
                self.dtime_gy_old = self.dtime_gy;
                self.dtime_min = DEVICES[self.fifo_ver as usize].dtime[FifoData::bdr_get_index(&DEVICES[self.fifo_ver as usize].bdr_acc, bdr_max)];
                self.dtime_xl = DEVICES[self.fifo_ver as usize].dtime[FifoData::bdr_get_index(&DEVICES[self.fifo_ver as usize].bdr_acc, bdr_xl)];
                self.dtime_gy = DEVICES[self.fifo_ver as usize].dtime[FifoData::bdr_get_index(&DEVICES[self.fifo_ver as usize].bdr_gyr, bdr_gy)];

                self.bdr_chg_xl_flag = 1;
                self.bdr_chg_gy_flag = 1;
            } else if tag == Tag::Ts {
                self.timestamp = u32::from_le_bytes(fifo_raw_slot[i].fifo_data_out[1..5].try_into().unwrap());
            } else {
                let compression_type = Self::get_compression_type(&tag);
                let sensor_type = Self::get_sensor_type(&tag);

                match compression_type {
                    CompressionType::Nc => {
                        if tag == Tag::Empty {
                            continue;
                        }

                        if tag == Tag::StepCounter || tag == Tag::MlcResult {
                            fifo_out_slot[j].timestamp = u32::from_le_bytes(fifo_raw_slot[i].fifo_data_out[3..7].try_into().unwrap());
                        } else {
                            fifo_out_slot[j].timestamp = self.timestamp;
                        }

                        fifo_out_slot[j].sensor_tag = sensor_type.clone();
                        fifo_out_slot[j].sensor_data.from_u8_arr(&fifo_raw_slot[i].fifo_data_out[1..7]);

                        if sensor_type == SensorType::Accelerometer {
                            self.last_data_xl = fifo_out_slot[j].sensor_data.data;
                            self.last_timestamp_xl = self.timestamp;
                            self.bdr_chg_xl_flag = 0;
                        }

                        if sensor_type == SensorType::Gyroscope {
                            self.last_data_gy = fifo_out_slot[j].sensor_data.data;
                            self.last_timestamp_gy = self.timestamp;
                            self.bdr_chg_gy_flag = 0;
                        }

                        j += 1;
                    }
                    CompressionType::NcT1 => {
                        fifo_out_slot[j].sensor_tag = sensor_type.clone();
                        fifo_out_slot[j].sensor_data.from_u8_arr(&fifo_raw_slot[i].fifo_data_out[1..7]);

                        if sensor_type == SensorType::Accelerometer {
                            let last_timestamp = if self.bdr_chg_xl_flag == 1 {
                                self.last_timestamp_xl + self.dtime_xl_old
                            } else {
                                self.timestamp - self.dtime_xl
                            };

                            fifo_out_slot[j].timestamp = last_timestamp;
                            self.last_data_xl = fifo_out_slot[j].sensor_data.data;
                            self.last_timestamp_xl = last_timestamp;
                        }

                        if sensor_type == SensorType::Gyroscope {
                            let last_timestamp = if self.bdr_chg_gy_flag == 1 {
                                self.last_timestamp_gy + self.dtime_gy_old
                            } else {
                                self.timestamp - self.dtime_gy
                            };

                            fifo_out_slot[j].timestamp = last_timestamp;
                            self.last_data_gy = fifo_out_slot[j].sensor_data.data;
                            self.last_timestamp_gy = last_timestamp;
                        }

                        j += 1;
                    }
                    CompressionType::NcT2 => {
                        fifo_out_slot[j].sensor_tag = sensor_type.clone();
                        fifo_out_slot[j].sensor_data.from_u8_arr(&fifo_raw_slot[i].fifo_data_out[1..7]);

                        if sensor_type == SensorType::Accelerometer {
                            let last_timestamp = if self.bdr_chg_xl_flag == 1 {
                                self.last_timestamp_xl + self.dtime_xl_old
                            } else {
                                self.timestamp - 2 * self.dtime_xl
                            };

                            fifo_out_slot[j].timestamp = last_timestamp;
                            self.last_data_xl = fifo_out_slot[j].sensor_data.data;
                            self.last_timestamp_xl = last_timestamp;
                        }

                        if sensor_type == SensorType::Gyroscope {
                            let last_timestamp = if self.bdr_chg_gy_flag == 1 {
                                self.last_timestamp_gy + self.dtime_gy_old
                            } else {
                                self.timestamp - 2 * self.dtime_gy
                            };

                            fifo_out_slot[j].timestamp = last_timestamp;
                            self.last_data_gy = fifo_out_slot[j].sensor_data.data;
                            self.last_timestamp_gy = last_timestamp;
                        }

                        j += 1;
                    }
                    CompressionType::Comp2x => {
                        let mut diff = [0i16; 6];
                        FifoData::get_diff_2x(&mut diff, &fifo_raw_slot[i].fifo_data_out[1..7]);

                        fifo_out_slot[j].sensor_tag = sensor_type.clone();

                        if sensor_type == SensorType::Accelerometer {
                            let data = [
                                self.last_data_xl[0] + diff[0],
                                self.last_data_xl[1] + diff[1],
                                self.last_data_xl[2] + diff[2],
                            ];
                            fifo_out_slot[j].timestamp = self.timestamp - 2 * self.dtime_xl;
                            self.last_data_xl = data;
                            fifo_out_slot[j].sensor_data.data = data;
                        }

                        if sensor_type == SensorType::Gyroscope {
                            let data =[
                                self.last_data_gy[0] + diff[0],
                                self.last_data_gy[1] + diff[1],
                                self.last_data_gy[2] + diff[2],
                            ];
                            fifo_out_slot[j].timestamp = self.timestamp - 2 * self.dtime_gy;
                            self.last_data_gy = data;
                            fifo_out_slot[j].sensor_data.data = data
                        }

                        j += 1;

                        fifo_out_slot[j].sensor_tag = sensor_type.clone();

                        if sensor_type == SensorType::Accelerometer {
                            let last_timestamp = self.timestamp - self.dtime_xl;
                            let data = [
                                self.last_data_xl[0] + diff[3],
                                self.last_data_xl[1] + diff[4],
                                self.last_data_xl[2] + diff[5],
                            ];
                            fifo_out_slot[j].timestamp = last_timestamp;
                            self.last_data_xl = data;
                            fifo_out_slot[j].sensor_data.data = data;
                            self.last_timestamp_xl = last_timestamp;
                        }

                        if sensor_type == SensorType::Gyroscope {
                            let last_timestamp = self.timestamp - self.dtime_gy;
                            let data = [
                                self.last_data_gy[0] + diff[3],
                                self.last_data_gy[1] + diff[4],
                                self.last_data_gy[2] + diff[5],
                            ];
                            fifo_out_slot[j].timestamp = last_timestamp;
                            self.last_data_gy = data;
                            fifo_out_slot[j].sensor_data.data = data;
                            self.last_timestamp_gy = last_timestamp;
                        }

                        j += 1;
                    }
                    CompressionType::Comp3x => {
                        let mut diff = [0i16; 9];
                        Self::get_diff_3x(&mut diff, &fifo_raw_slot[i].fifo_data_out[1..7]);

                        fifo_out_slot[j].sensor_tag = sensor_type.clone();

                        if sensor_type == SensorType::Accelerometer {
                            let data = [
                                self.last_data_xl[0] + diff[0],
                                self.last_data_xl[1] + diff[1],
                                self.last_data_xl[2] + diff[2],
                            ];
                            fifo_out_slot[j].timestamp = self.timestamp - 2 * self.dtime_xl;
                            self.last_data_xl = data;
                            fifo_out_slot[j].sensor_data.data = data
                        }

                        if sensor_type == SensorType::Gyroscope {
                            let data = [
                                self.last_data_gy[0] + diff[0],
                                self.last_data_gy[1] + diff[1],
                                self.last_data_gy[2] + diff[2],
                            ];
                            fifo_out_slot[j].timestamp = self.timestamp - 2 * self.dtime_gy;
                            self.last_data_gy = data;
                            fifo_out_slot[j].sensor_data.data = data;
                        }

                        j += 1;


                        if sensor_type == SensorType::Accelerometer {
                            let data = [
                                self.last_data_xl[0] + diff[3],
                                self.last_data_xl[1] + diff[4],
                                self.last_data_xl[2] + diff[5],
                            ];
                            fifo_out_slot[j].sensor_data.data = data;
                            self.last_data_xl = data;
                            fifo_out_slot[j].timestamp = self.timestamp - self.dtime_xl;
                            self.last_timestamp_xl = self.timestamp;
                        }

                        if sensor_type == SensorType::Gyroscope {
                            let data = [
                                self.last_data_gy[0] + diff[3],
                                self.last_data_gy[1] + diff[4],
                                self.last_data_gy[2] + diff[5],
                            ];
                            fifo_out_slot[j].sensor_data.data = data; 
                            self.last_data_gy = data;
                            fifo_out_slot[j].timestamp = self.timestamp - self.dtime_gy;
                            self.last_timestamp_gy = self.timestamp;
                        }

                        j += 1;

                        fifo_out_slot[j].timestamp = self.timestamp;
                        fifo_out_slot[j].sensor_tag = sensor_type.clone();

                        if sensor_type == SensorType::Accelerometer {
                            let data = [
                                self.last_data_xl[0] + diff[6],
                                self.last_data_xl[1] + diff[7],
                                self.last_data_xl[2] + diff[8],
                            ];
                            self.last_data_xl = data;
                            fifo_out_slot[j].sensor_data.data = data;
                            self.last_timestamp_xl = self.timestamp;
                        }

                        if sensor_type == SensorType::Gyroscope {
                            let data = [
                                self.last_data_gy[0] + diff[6],
                                self.last_data_gy[1] + diff[7],
                                self.last_data_gy[2] + diff[8],
                            ];
                            self.last_data_gy = data;
                            fifo_out_slot[j].sensor_data.data = data;
                            self.last_timestamp_gy = self.timestamp;
                        }

                        j += 1;
                    }
                }

                *out_slot_size = j as u16;
            }

            self.tag_counter_old = tag_counter;
        }

        Status::Ok
    }
    pub fn bytes_to_i16_array(source_bytes: &[u8; 6], destination: &mut [i16; 3]) {
        for (i, chunk) in source_bytes.chunks_exact(2).enumerate() {
            destination[i] = FifoData::combine_bytes_to_i16(chunk[0], chunk[1]);
        }
    }

    pub fn combine_bytes_to_i16(low_byte: u8, high_byte: u8) -> i16 {
        (((low_byte as u16) << 8) | high_byte as u16) as i16
    }

    
    fn is_tag_valid(&self, tag: u8) -> bool {
        tag <= DEVICES[self.fifo_ver as usize].tag_valid_limit
    }

    fn get_sensor_type(tag: &Tag) -> SensorType {
        match tag {
            Tag::Gy => SensorType::Gyroscope,
            Tag::Xl => SensorType::Accelerometer,
            Tag::Temp => SensorType::Temperature,
            Tag::ExtSens0 => SensorType::ExtSensor0,
            Tag::ExtSens1 => SensorType::ExtSensor1,
            Tag::ExtSens2 => SensorType::ExtSensor2,
            Tag::ExtSens3 => SensorType::ExtSensor3,
            Tag::XlUncompressedT2 => SensorType::Accelerometer,
            Tag::XlUncompressedT1 => SensorType::Accelerometer,
            Tag::XlCompressed2x => SensorType::Accelerometer,
            Tag::XlCompressed3x => SensorType::Accelerometer,
            Tag::GyUncompressedT2 => SensorType::Gyroscope,
            Tag::GyUncompressedT1 => SensorType::Gyroscope,
            Tag::GyCompressed2x => SensorType::Gyroscope,
            Tag::GyCompressed3x => SensorType::Gyroscope,
            Tag::StepCounter => SensorType::StepCounter,
            Tag::GameRv => SensorType::GameRv6x,
            Tag::GeomRv => SensorType::GeomRv6x,
            Tag::NormRv => SensorType::Rv9x,
            Tag::GyroBias => SensorType::GyroBias,
            Tag::Gravity => SensorType::Gravity,
            Tag::MagCal => SensorType::MagCalib,
            Tag::ExtSensNack => SensorType::ExtSensorNack,
            Tag::MlcResult => SensorType::MlcResult,
            Tag::MlcFilter => SensorType::MlcFilter,
            Tag::MlcFeature => SensorType::MlcFeature,
            Tag::DualcXl => SensorType::DualAccel,
            Tag::EisGy => SensorType::EisGyro,
            _ => SensorType::None,
        }
    }

    fn get_compression_type(tag: &Tag) -> CompressionType {
        match tag {
            Tag::XlUncompressedT2 | Tag::GyUncompressedT2 => CompressionType::NcT2,
            Tag::XlUncompressedT1 | Tag::GyUncompressedT1 => CompressionType::NcT1,
            Tag::XlCompressed2x | Tag::GyCompressed2x => CompressionType::Comp2x,
            Tag::XlCompressed3x | Tag::GyCompressed3x => CompressionType::Comp3x,
            _ => CompressionType::Nc,
        }
    }

    fn bdr_get_index(bdr: &[f32; 16], n: f32) -> usize {
        let mut min_diff = f32::MAX;
        let mut idx = 0;

        for (i, &value) in bdr.iter().enumerate() {
            let diff = (value - n).abs();
            if diff < min_diff {
                min_diff = diff;
                idx = i;
            }
        }

        idx
    }

    fn has_even_parity(x: u8) -> bool {
        x.count_ones() % 2 == 0
    }

    fn get_diff_2x(diff: &mut [i16; 6], input: &[u8]) {
        for (i, &byte) in input.iter().enumerate() {
            diff[i] = if byte < 128 { byte as i16 } else { byte as i16 - 256 };
        }
    }

    fn get_diff_3x(diff: &mut [i16; 9], input: &[u8]) {
        for i in 0..3 {
            let decode_tmp = u16::from_le_bytes([input[2 * i], input[2 * i + 1]]);
            for j in 0..3 {
                let utmp = (decode_tmp >> (5 * j)) & 0x1F;
                let tmp = utmp as i16;
                diff[j + 3 * i] = if tmp < 16 { tmp } else { tmp - 32 };
            }
        }
    }


    pub fn sort(&self, fifo_out_slot: &mut [OutSlot], out_slot_size: u16) {
        for i in 1..out_slot_size as usize {
            let temp = fifo_out_slot[i].clone();
            let mut j: i32 = i as i32 - 1;

            while j >= 0 && fifo_out_slot[j as usize].timestamp > temp.timestamp {
                fifo_out_slot[(j + 1) as usize] = fifo_out_slot[j as usize].clone();
                j -= 1;
            }

            fifo_out_slot[(j + 1) as usize] = temp;
        }
    }

    pub fn get_sensor_occurrence(&self, fifo_out_slot: &[OutSlot], out_slot_size: u16, sensor_type: SensorType) -> u16 {
        let mut occurrence = 0;

        for i in 0..out_slot_size as usize {
            if fifo_out_slot[i].sensor_tag == sensor_type {
                occurrence += 1;
            }
        }

        occurrence
    }

    pub fn extract_sensor(&self, sensor_out_slot: &mut [OutSlot], fifo_out_slot: &[OutSlot], out_slot_size: u16, sensor_type: SensorType) {
        let mut temp_i = 0;

        for i in 0..out_slot_size as usize {
            if fifo_out_slot[i].sensor_tag == sensor_type {
                sensor_out_slot[temp_i] = fifo_out_slot[i].clone();
                temp_i += 1;
            }
        }
    }

}



#[repr(u8)]
pub enum CompressionType {
    Nc,
    NcT1,
    NcT2,
    Comp2x,
    Comp3x,
}

pub struct Device {
    pub bdr_acc: [f32; 16],
    pub bdr_gyr: [f32; 16],
    pub bdr_vsens: [f32; 16],
    pub dtime: [u32; 16],
    pub tag_valid_limit: u8,
}

#[derive(PartialEq)]
pub enum BdrMask {
    Xl,
    Gy,
    Vsens,
}

impl From<BdrMask> for u8 {
    fn from(value: BdrMask) -> Self {
        match value {
            BdrMask::Xl => 0x0F,
            BdrMask::Gy => 0xF0,
            BdrMask::Vsens => 0x0F,
        }
    }
}

#[derive(PartialEq)]
pub enum BdrShift {
    Xl,
    Gy,
    Vsens,
}

impl From<BdrShift> for u8 {
    fn from(value: BdrShift) -> Self {
        match value {
            BdrShift::Xl => 0x00,
            BdrShift::Gy => 0x04,
            BdrShift::Vsens => 0x00,
        }
    }
}

#[derive(PartialEq)]
#[repr(u8)]
pub enum TagMask {
    Counter = 0x06,
    Sensor = 0xF8,
}

#[derive(PartialEq)]
#[repr(u8)]
pub enum TagShift {
    Counter = 0x01,
    Sensor = 0x03,
}

#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum Status {
    Ok = 0,
    Err = 1,
}


#[derive(PartialEq, Clone)]
#[repr(u8)]
pub enum Tag {
    Empty = 0x00,
    Gy = 0x01,
    Xl = 0x02,
    Temp = 0x03,
    Ts = 0x04,
    Odrchg = 0x05,
    XlUncompressedT2 = 0x06,
    XlUncompressedT1 = 0x07,
    XlCompressed2x = 0x08,
    XlCompressed3x = 0x09,
    GyUncompressedT2 = 0x0A,
    GyUncompressedT1 = 0x0B,
    GyCompressed2x = 0x0C,
    GyCompressed3x = 0x0D,
    ExtSens0 = 0x0E,
    ExtSens1 = 0x0F,
    ExtSens2 = 0x10,
    ExtSens3 = 0x11,
    StepCounter = 0x12,
    GameRv = 0x13,
    GeomRv = 0x14,
    NormRv = 0x15,
    GyroBias = 0x16,
    Gravity = 0x17,
    MagCal = 0x18,
    ExtSensNack = 0x19,
    MlcResult = 0x1A,
    MlcFilter = 0x1B,
    MlcFeature = 0x1C,
    DualcXl = 0x1D,
    EisGy = 0x1E,
}

impl TryFrom<u8> for Tag {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Tag::Empty),
            0x01 => Ok(Tag::Gy),
            0x02 => Ok(Tag::Xl),
            0x03 => Ok(Tag::Temp),
            0x04 => Ok(Tag::Ts),
            0x05 => Ok(Tag::Odrchg),
            0x06 => Ok(Tag::XlUncompressedT2),
            0x07 => Ok(Tag::XlUncompressedT1),
            0x08 => Ok(Tag::XlCompressed2x),
            0x09 => Ok(Tag::XlCompressed3x),
            0x0A => Ok(Tag::GyUncompressedT2),
            0x0B => Ok(Tag::GyUncompressedT1),
            0x0C => Ok(Tag::GyCompressed2x),
            0x0D => Ok(Tag::GyCompressed3x),
            0x0E => Ok(Tag::ExtSens0),
            0x0F => Ok(Tag::ExtSens1),
            0x10 => Ok(Tag::ExtSens2),
            0x11 => Ok(Tag::ExtSens3),
            0x12 => Ok(Tag::StepCounter),
            0x13 => Ok(Tag::GameRv),
            0x14 => Ok(Tag::GeomRv),
            0x15 => Ok(Tag::NormRv),
            0x16 => Ok(Tag::GyroBias),
            0x17 => Ok(Tag::Gravity),
            0x18 => Ok(Tag::MagCal),
            0x19 => Ok(Tag::ExtSensNack),
            0x1A => Ok(Tag::MlcResult),
            0x1B => Ok(Tag::MlcFilter),
            0x1C => Ok(Tag::MlcFeature),
            0x1D => Ok(Tag::DualcXl),
            0x1E => Ok(Tag::EisGy),
            _ => Err(()), // Return an error for unknown values
        }
    }
}

#[derive(PartialEq, Clone, Copy, Default)]
#[repr(u8)]
pub enum SensorType {
    #[default] Gyroscope = 0,
    Accelerometer = 1,
    Temperature = 2,
    ExtSensor0 = 3,
    ExtSensor1 = 4,
    ExtSensor2 = 5,
    ExtSensor3 = 6,
    StepCounter = 7,
    GameRv6x = 8,
    GeomRv6x = 9,
    Rv9x = 10,
    GyroBias = 11,
    Gravity = 12,
    MagCalib = 13,
    ExtSensorNack = 14,
    MlcResult = 15,
    MlcFilter = 16,
    MlcFeature = 17,
    DualAccel = 18,
    EisGyro = 19,
    None = 20,
}

impl fmt::Display for SensorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            SensorType::Gyroscope => "Gyroscope",
            SensorType::Accelerometer => "Accelerometer",
            SensorType::Temperature => "Temperature",
            SensorType::ExtSensor0 => "ExtSensor0",
            SensorType::ExtSensor1 => "ExtSensor1",
            SensorType::ExtSensor2 => "ExtSensor2",
            SensorType::ExtSensor3 => "ExtSensor3",
            SensorType::StepCounter => "StepCounter",
            SensorType::GameRv6x => "GameRv6x",
            SensorType::GeomRv6x => "GeomRv6x",
            SensorType::Rv9x => "Rv9x",
            SensorType::GyroBias => "GyroBias",
            SensorType::Gravity => "Gravity",
            SensorType::MagCalib => "MagCalib",
            SensorType::ExtSensorNack => "ExtSensorNack",
            SensorType::MlcResult => "MlcResult",
            SensorType::MlcFilter => "MlcFilter",
            SensorType::MlcFeature => "MlcFeature",
            SensorType::DualAccel => "DualAccel",
            SensorType::EisGyro => "EisGyro",
            SensorType::None => "None",
        };
        write!(f, "{}", name)
    }
}

#[derive(Default, Copy, Clone)]
pub struct RawSlot {
    pub fifo_data_out: [u8; 7], // registers from mems (78h -> 7Dh)
}

#[derive(Clone, Copy, Default)]
pub struct OutSlot {
    pub timestamp: u32,
    pub sensor_tag: SensorType,
    pub sensor_data: SensorData,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DeviceType {
    Lsm6dsr = 0,
    Lsm6dsrx = 1,
    Asm330lhh = 2,
    Asm330lhhx = 3,
    Ism330dhcx = 4,
    Lsm6dso = 5,
    Lsm6dsox = 6,
    Lsm6dso32 = 7,
    Lsm6dso32x = 8,
    Lsm6dsv = 9,
    Lsm6dsv16x = 10,
    Lsm6dsv32x = 11,
}

pub struct Config {
    pub device: DeviceType, // device to select
    pub bdr_xl: f32,    // accelerometer batch data rate in Hz
    pub bdr_gy: f32,    // gyroscope batch data rate in Hz
    pub bdr_vsens: f32, // virtual sensor batch data rate in Hz
}
