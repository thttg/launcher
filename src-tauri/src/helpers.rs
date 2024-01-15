use std::fs;
use std::path::Path;
use std::borrow::Cow;

use chardet::{charset2encoding, detect};
use charset_normalizer_rs::from_bytes;
use encoding_rs::*;
use log::info;

pub fn decode_buffer(buf: Vec<u8>) -> (String, String, String) {
    let first_encoding: String;
    let second_encoding: String;
    let mut buff_output: String;
    let mut str_encoding: String;

    // chardet
    first_encoding = charset2encoding(&detect(&buf).0).to_string();

    // charset_normalizer_rs
    second_encoding = match from_bytes(&buf, None).get_best() {
        Some(cd) => cd.encoding().to_string(),
        None => "not_found".to_string(),
    };

    // First try using first_encoding
    if let Some(encoding) = Encoding::for_label(first_encoding.as_bytes()) {
        buff_output = match encoding.decode(&buf).0 {
            Cow::Owned(s) => s,
            Cow::Borrowed(s) => s.to_string(),
        };
    } else if let Some(encoding) = Encoding::for_label(second_encoding.as_bytes()) {
        // If first_encoding fails, try second_encoding
        buff_output = match encoding.decode(&buf).0 {
            Cow::Owned(s) => s,
            Cow::Borrowed(s) => s.to_string(),
        };
    } else {
        buff_output = String::from_utf8_lossy(&buf).to_string();
    }

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
