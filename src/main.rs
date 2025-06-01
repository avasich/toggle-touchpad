use std::{
    fs::{DirEntry, File, ReadDir},
    io::{BufRead, BufReader},
    path::Path,
};

fn find_entry(entry: DirEntry, predicate: &impl Fn(&Path) -> bool) -> Option<DirEntry> {
    let metadata = entry.metadata().ok()?;
    if metadata.is_dir() {
        traverse_dir(std::fs::read_dir(entry.path()).ok()?, predicate)
    } else if metadata.is_file() && predicate(entry.path().as_path()) {
        Some(entry)
    } else {
        None
    }
}

fn traverse_dir(dir: ReadDir, predicate: &impl Fn(&Path) -> bool) -> Option<DirEntry> {
    dir.flatten().filter_map(|entry| find_entry(entry, predicate)).next()
}

fn main() -> Result<(), std::io::Error> {
    let pci_dir = std::fs::read_dir("/sys/devices/pci0000:00")?;
    let inhibited_path = traverse_dir(pci_dir, &|path| {
        File::open(path)
            .ok()
            .and_then(|f| BufReader::new(f).lines().next())
            .and_then(Result::ok)
            .is_some_and(|line| line.contains("Touchpad"))
    })
    .and_then(|entry| entry.path().parent().map(|parent| parent.join("inhibited")));

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
