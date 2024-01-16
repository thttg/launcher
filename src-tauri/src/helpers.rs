use std::fs;
use std::path::Path;

use encoding_rs::{Encoding, UTF_8};
use log::info;

pub fn decode_buffer(buf: Vec<u8>) -> (String, String, String) {
    let (encoding, offset) = Encoding::for_bom(&buf).unwrap_or((UTF_8, 0));
    let first_encoding = encoding.name().to_string();
    let second_encoding = first_encoding.clone();

    let (decoded, _, had_errors) = encoding.decode(&buf[offset..]);
    let buff_output = if had_errors {
        String::from_utf8_lossy(&buf).into_owned()
    } else {
        decoded.into_owned()
    };

    (buff_output, first_encoding, second_encoding)
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
