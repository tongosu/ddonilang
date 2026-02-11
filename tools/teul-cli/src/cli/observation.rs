use std::fs;
use std::path::Path;

use ddonirang_core::seulgi::observation::{observation_detjson, observation_from_detjson};

use super::detjson::write_text;

pub fn run_canon(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let raw = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let obs = observation_from_detjson(&raw)?;
    let detjson = observation_detjson(&obs);
    if let Some(path) = out {
        write_text(path, &format!("{}\n", detjson))?;
    } else {
        println!("{}", detjson);
    }
    Ok(())
}
