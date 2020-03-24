use app_dirs::{AppDataType, AppInfo};
use std::fs;
use std::path::PathBuf;

use crate::error::{NoSuchWorld, WorldAlreadyExists};
use crate::version::Version;

const APP_INFO: AppInfo = AppInfo {
    name: "facts",
    author: "dento",
};

pub fn app_root() -> PathBuf {
    app_dirs::app_root(AppDataType::UserData, &APP_INFO).expect("No data dir available")
}

/// Creates directory `worlds/$name` and required subdirectories
pub fn new_world(name: &str) -> Result<PathBuf, WorldAlreadyExists> {
    let mut pb = app_root();
    pb.push("worlds");

    fs::create_dir_all(&pb).expect("Could not create dir");

    pb.push(app_dirs::sanitized(name));

    if pb.exists() {
        Err(WorldAlreadyExists(name.to_owned()))
    } else {
        fs::create_dir(&pb).expect("Could not create dir");
        // Create subdirectories
        fs::create_dir(&pb.join("factorio")).expect("Could not create dir");
        fs::create_dir(&pb.join("factorio/mods")).expect("Could not create dir");
        Ok(pb)
    }
}

/// Returns directory `worlds/$name` if it exists
pub fn get_world(name: &str) -> Result<PathBuf, NoSuchWorld> {
    let mut pb = app_root();
    pb.push("worlds");
    pb.push(app_dirs::sanitized(name));

    if pb.exists() {
        Ok(pb)
    } else {
        Err(NoSuchWorld(name.to_owned()))
    }
}

/// Returns all folders under `worlds/`
pub fn list_worlds() -> Vec<String> {
    let mut pb = app_root();
    pb.push("worlds");

    if let Ok(paths) = fs::read_dir(pb) {
        paths
            .map(|p| String::from(p.unwrap().path().file_name().unwrap().to_str().unwrap()))
            .collect()
    } else {
        Vec::new()
    }
}

/// Returns all downloaded versions
pub fn list_versions() -> Vec<Version> {
    let mut pb = app_root();
    pb.push("versions");

    if let Ok(paths) = fs::read_dir(pb) {
        paths
            .map(|p| {
                Version::from_str(&String::from(
                    p.unwrap().path().file_name().unwrap().to_str().unwrap(),
                ))
            })
            .collect()
    } else {
        Vec::new()
    }
}

pub fn version_data(version: Version) -> Result<PathBuf, PathBuf> {
    let mut pb = app_root();
    pb.push("versions");

    fs::create_dir_all(&pb).expect("Could not create dir");

    pb.push(version.to_string());

    if pb.exists() { Ok(pb) } else { Err(pb) }
}

pub fn delete_version(version: Version) {
    let mut pb = app_root();
    pb.push("versions");
    pb.push(version.to_string());

    fs::remove_dir_all(&pb).expect("Could not delete dir");
}
