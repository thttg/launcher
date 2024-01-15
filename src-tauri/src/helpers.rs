use std::fs;
use std::path::Path;

use chardet::{charset2encoding, detect};
use encoding::label::encoding_from_whatwg_label;
use encoding::DecoderTrap;
use encoding_rs::Encoding;
use log::info;

pub fn decode_buffer(buf: Vec<u8>) -> Result<String, String> {
    // First attempt with chardet
    let detected_encoding = charset2encoding(&detect(&buf).0);
    if let Some(encoder) = encoding_from_whatwg_label(detected_encoding) {
        if let Ok(decoded_string) = encoder.decode(&buf, DecoderTrap::Replace) {
            return Ok(decoded_string);
        }
    }

    // Second attempt with encoding_rs
    if let Some((encoding, _, _)) = Encoding::for_bom(&buf) {
        if let Ok(decoded_string) = encoding.decode_without_bom_handling_and_without_replacement(&buf) {
            return Ok(decoded_string.into_owned());
        }
    }

    // Default to UTF-8 as a fallback
    match String::from_utf8(buf) {
        Ok(s) => Ok(s),
        Err(e) => Err(format!("Failed to decode buffer: {}", e)),
    }
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
