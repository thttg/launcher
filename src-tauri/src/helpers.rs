use std::fs;
use std::path::Path;

use chardetng::EncodingDetector;
use encoding_rs::{Encoding, UTF_8};
use log::info;

pub fn decode_buffer(buf: Vec<u8>) -> (String, String, String) {
    // Use chardetng for preliminary encoding detection
    let mut detector = EncodingDetector::new();
    detector.feed(&buf, true);
    let guessed_encoding = detector.guess(None, true).name();

    // Modify detection to handle specific encodings as cp1251
    let encoding = match guessed_encoding {
        "KOI8-R" | "MacCyrillic" | "x-mac-cyrillic" | "koi8-r" | "macintosh" | "ibm866" => "windows-1251",
        _ => guessed_encoding,
    };

    // Use encoding_rs for decoding
    let actual_encoding = Encoding::for_label(encoding.as_bytes()).unwrap_or(UTF_8);
    let (decoded, _, had_errors) = actual_encoding.decode(&buf);

    let buff_output = if had_errors {
        String::from_utf8_lossy(&buf).into_owned()
    } else {
        decoded.into_owned()
    };

    let first_encoding = actual_encoding.name().to_string();
    let second_encoding = first_encoding.clone();

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
