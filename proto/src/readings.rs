use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Imu {
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
}
