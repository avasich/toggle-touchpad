use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};


fn main() -> Result<(), std::io::Error> {
    let device_name = "Touchpad";
    if let Some(path) = find_in_class_input(device_name)
        .or_else(|| find_device(Path::new("/sys/devices"), device_name, &mut HashSet::new()))
    {
        toggle_inhibited(&path)?;
    }
    Ok(())
}


/// looks for corresponding 'inhibited' file in /sys/class/input/input*
fn find_in_class_input(device_name: &str) -> Option<PathBuf> {
    std::fs::read_dir("/sys/class/input")
        .ok()?
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("input"))
        .map(|entry| entry.path())
        .find_map(|path| check_directory(&path, device_name))
}


/// the directory should contain a 'name' file with the device name and an 'inhibited' file we need
fn check_directory(path: &Path, device_name: &str) -> Option<PathBuf> {
    if let Ok(name_line) = std::fs::read_to_string(path.join("name"))
        && name_line.contains(device_name)
        && let inhibited_path = path.join("inhibited")
        && matches!(std::fs::exists(&inhibited_path), Ok(true))
    {
        Some(inhibited_path)
    } else {
        None
    }
}


/// we haven't found the device under /sys/class/input and are defaulting to full /sys/devices traversal.
fn find_device(path: &Path, device_name: &str, visited: &mut HashSet<PathBuf>) -> Option<PathBuf> {
    if !path.is_dir() {
        return None;
    }

    let path = std::fs::canonicalize(path).ok()?;
    if !visited.insert(path.clone()) {
        return None;
    }

    check_directory(&path, device_name).or_else(|| {
        std::fs::read_dir(path).ok()?.flatten().find_map(|entry| find_device(&entry.path(), device_name, visited))
    })
}


fn toggle_inhibited(path: &Path) -> Result<(), std::io::Error> {
    let new_value = match std::fs::read_to_string(path)?.trim() {
        "0" => "1",
        "1" => "0",
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid inhibited file content")),
    };
    std::fs::write(path, new_value)
}
