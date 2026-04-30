use super::{file::FileData, resources};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Flag {
    rel_offset: usize,
    dead_value: u8,
    alive_value: u8,
    current_value: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Boss {
    name: String,
    flags: Vec<Flag>,
}

pub fn new(file: &FileData) -> Result<Vec<Boss>, io::Error> {
    let raw = resources::read(file, "bosses.json")?;
    let bosses: Vec<Boss> = serde_json::from_str(&raw)?;
    Ok(populate(file, bosses))
}

pub fn from_json(file: &FileData, json: &str) -> Result<Vec<Boss>, serde_json::Error> {
    let bosses: Vec<Boss> = serde_json::from_str(json)?;
    Ok(populate(file, bosses))
}

fn populate(file: &FileData, mut bosses: Vec<Boss>) -> Vec<Boss> {
    for b in &mut bosses {
        for f in &mut b.flags {
            f.current_value = file.get_flag(f.rel_offset);
        }
    }
    bosses
}
