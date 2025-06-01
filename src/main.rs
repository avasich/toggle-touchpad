fn main() -> Result<(), std::io::Error> {
    let inhibited_path = std::fs::read_dir("/sys/class/input")?.flatten().find_map(|entry| {
        let name_path = entry.path().join("device/name");
        std::fs::read_to_string(&name_path).ok().filter(|line| line.to_lowercase().contains("touchpad")).map(|_| entry.path().join("device/inhibited"))
    });

    if let Some(inhibited_path) = inhibited_path {
        let contents = std::fs::read_to_string(&inhibited_path)?.trim().to_string();
        let new_value = match contents.as_str() {
            "0" => "1",
            "1" => "0",
            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid content")),
        };
        std::fs::write(&inhibited_path, new_value)?;
    }
    Ok(())
}
