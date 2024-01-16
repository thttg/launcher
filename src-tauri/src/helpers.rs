use std::fs;
use std::path::Path;

use chardetng::EncodingDetector;
use encoding_rs::{Encoding, UTF_8, WINDOWS_1251};
use log::info;

pub fn decode_buffer(buf: Vec<u8>) -> (String, String, String) {
    let mut detector = EncodingDetector::new();
    detector.feed(&buf, true);
    let guessed_encoding = detector.guess(None, true).name();

    let (decoded, first_encoding) = try_decode(&buf, guessed_encoding)
        .or_else(|| try_decode(&buf, "windows-1251"))
        .unwrap_or_else(|| (String::from_utf8_lossy(&buf).into_owned(), "utf-8".to_string()));

    let second_encoding = first_encoding.clone();

    (decoded, first_encoding, second_encoding)
}

fn try_decode(buf: &[u8], encoding_name: &str) -> Option<(String, String)> {
    Encoding::for_label(encoding_name.as_bytes()).and_then(|encoding| {
        let (decoded, _, had_errors) = encoding.decode(buf);
        if had_errors {
            None
        } else {
            Some((decoded.into_owned(), encoding_name.to_string()))
        }
    })
}

pub fn copy_files(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<(), String> {
    let read_results = fs::read_dir(src);
    match read_results {
        Ok(files) => {
            for entry in files {
                match entry {
                    Ok(entry) => {
                        let ty = entry.file_type().unwrap();
                        if ty.is_dir() {
                            let dir_path = dest.as_ref().join(entry.file_name());
                            let dir_path_str = dir_path.to_str().unwrap();
                            let dir_creation_results = fs::create_dir(dir_path.to_owned());
                            match dir_creation_results {
                                Ok(_) => {}
                                Err(e) => {
                                    if e.raw_os_error().is_some() {
                                        if e.raw_os_error().unwrap() == 183 {
                                            println!("Directory {} already exists", dir_path_str)
                                        }
                                    } else {
                                        info!("[helpers.rs] copy_files: {}", e.to_string());
                                        return Err(e.to_string());
                                    }
                                }
                            }

                            match copy_files(entry.path(), dest.as_ref().join(entry.file_name())) {
                                Ok(_) => {}
                                Err(e) => {
                                    info!("[helpers.rs] copy_files: {}", e.to_string());
                                    return Err(e.to_string());
                                }
                            }
                        } else {
                            let copy_results =
                                fs::copy(entry.path(), dest.as_ref().join(entry.file_name()));
                            match copy_results {
                                Ok(_) => {}
                                Err(e) => {
                                    info!("[helpers.rs] copy_files: {}", e.to_string());
                                    return Err(e.to_string());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        info!("[helpers.rs] copy_files: {}", e.to_string());
                        return Err(e.to_string());
                    }
                }
            }
            return Ok(());
        }
        Err(e) => {
            info!("[helpers.rs] copy_files: {}", e.to_string());
            return Err(e.to_string());
        }
    }
}
