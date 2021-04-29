use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn load_files() -> HashMap<String, PathBuf> {
    let audio_resource_path = "resources/audio";
    let files_iter = fs::read_dir(audio_resource_path).unwrap();
    let files: HashMap<String, PathBuf> = files_iter
        .filter_map(|entry| {
            let entry_val = entry.ok()?;
            Some((
                String::from(entry_val.path().file_stem()?.to_str()?),
                entry_val.path(),
            ))
        })
        .collect();

    files
}
