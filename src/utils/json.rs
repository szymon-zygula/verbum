//! JSON serialization and deserialization utilities.
//!
//! This module provides convenience functions for loading and saving JSON files.

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use serde::{Serialize, de::DeserializeOwned};

/// Loads a value from a JSON file.
///
/// # Arguments
///
/// * `path` - The path to the JSON file
///
/// # Returns
///
/// Returns the deserialized value on success, or an error if the file
/// cannot be read or parsed
pub fn load_json<T, P>(path: P) -> Result<T, Box<dyn Error>>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let data = serde_json::from_str(&contents)?;
    Ok(data)
}

/// Saves a value to a JSON file with pretty formatting.
///
/// # Arguments
///
/// * `data` - The value to serialize
/// * `path` - The path where the JSON file should be saved
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if the file cannot be written
pub fn save_json<T, P>(data: &T, path: P) -> Result<(), Box<dyn Error>>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let serialized = serde_json::to_string_pretty(data)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}
