use std::fs::{
    File,
    OpenOptions
};
use std::io::{
    Read,
    Write
};
use std::path::Path;
use std::error::Error;

use serde::{
    Serialize,
    de::DeserializeOwned
};

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
