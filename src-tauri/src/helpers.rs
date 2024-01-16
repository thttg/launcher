use std::fs;
use std::path::Path;

use chardet::{charset2encoding, detect};
use chardetng::EncodingDetector;
use encoding_rs::{Encoding, UTF_8};
use log::info;

pub fn decode_buffer(buf: Vec<u8>) -> (String, String, String) {
    // Use chardetng for preliminary encoding detection
    let mut detector = EncodingDetector::new();
    detector.feed(&buf, true);
    let guessed_encoding = detector.guess(None, true).name();

    // Use chardet for encoding detection
    let first_encoding = charset2encoding(&detect(&buf).0).to_string();

    // Initialize variables
    let buff_output: String;
    let actual_encoding: &Encoding;

    // Modify detection to handle specific encodings as cp1251
    if first_encoding == "KOI8-R"
        || first_encoding == "MacCyrillic"
        || first_encoding == "x-mac-cyrillic"
    {
        actual_encoding = Encoding::for_label("cp1251".as_bytes()).unwrap_or(UTF_8);
    } else {
        actual_encoding = Encoding::for_label(guessed_encoding.as_bytes()).unwrap_or(UTF_8);
    }

    // Use encoding_rs for decoding
    let (decoded, _, had_errors) = actual_encoding.decode(&buf);

    buff_output = if had_errors {
        String::from_utf8_lossy(&buf).into_owned()
    } else {
        decoded.into_owned()
    };

    let second_encoding = actual_encoding.name().to_string();

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
