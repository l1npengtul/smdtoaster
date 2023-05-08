use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Profile {
    pub name: [u8; 6],
    pub points: [CurvePoint; 6],
}

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CurvePoint {
    pub temp: u16,
    pub time_seconds: u16,
    pub disabled: bool
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Profiles {
    pub num_profiles: u8,
    pub profiles: [Option<Profile>; 16]
}