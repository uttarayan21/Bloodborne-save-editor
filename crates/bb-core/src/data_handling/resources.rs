use std::sync::OnceLock;

static RESOURCES: OnceLock<&'static [(&'static str, &'static str)]> = OnceLock::new();

/// Register an in-memory resource set. Intended for environments without a
/// filesystem (e.g. wasm). Disk lookups are used as a fallback when no entry
/// is registered for a given name.
pub fn register(entries: &'static [(&'static str, &'static str)]) {
    let _ = RESOURCES.set(entries);
}

pub fn get(name: &str) -> Option<&'static str> {
    RESOURCES
        .get()
        .and_then(|entries| entries.iter().find(|(k, _)| *k == name).map(|(_, v)| *v))
}

pub fn read(file_data: &super::file::FileData, name: &str) -> std::io::Result<String> {
    read_named(&file_data.resources_path, name)
}

pub fn read_named(resources_path: &std::path::Path, name: &str) -> std::io::Result<String> {
    if let Some(s) = get(name) {
        return Ok(s.to_string());
    }
    std::fs::read_to_string(resources_path.join(name))
}
