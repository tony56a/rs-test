use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use uuid::Uuid;

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

pub fn generate_tts_file(text: &str) -> Option<PathBuf> {
    let file_id = Uuid::new_v4().to_hyphenated().to_string();
    let path = format!("/tmp/{}.wav", file_id);

    let args = ["-w", path.as_str(), "-s", "100", "-a", "150", text];

    let result = Command::new("espeak")
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Some(PathBuf::from(path))
            } else {
                println!("{}", String::from_utf8(output.stderr).unwrap());
                println!("status: {}", output.status.code().unwrap());
                None
            }
        }
        Err(e) => {
            println!("error: {}", e);
            None
        }
    }
}

pub fn combine_files(first_file: &PathBuf, second_file: &PathBuf) -> Option<PathBuf> {
    let output_file_name = format!("/tmp/{}.mp3", second_file.file_stem()?.to_str()?);

    let args = [
        "-i",
        first_file.to_str().unwrap(),
        "-i",
        second_file.to_str().unwrap(),
        "-filter_complex",
        "[0:0][1:0]concat=n=2:v=0:a=1[out]",
        "-map",
        "[out]",
        &output_file_name,
    ];

    let result = Command::new("ffmpeg")
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Some(PathBuf::from(&output_file_name))
            } else {
                println!("error: {}", String::from_utf8(output.stderr).unwrap());
                println!("status: {}", output.status.code().unwrap());
                None
            }
        }
        Err(e) => {
            println!("error: {}", e);
            None
        }
    }
}
