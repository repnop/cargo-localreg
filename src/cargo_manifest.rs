use cargo::{
    core::{enable_nightly_features, Workspace},
    CargoError, Config,
};
use serde_json;
use std::env::current_dir;

pub fn generate_registry_json() -> Result<String, CargoError> {
    enable_nightly_features();
    let mut curr_dir = current_dir().unwrap();
    curr_dir.push("Cargo.toml");
    let cfg = Config::default().unwrap();
    let ws = Workspace::new(&curr_dir, &cfg)?;

    Ok(serde_json::to_string(&ws.generate_index_metadata()?).unwrap())
}
