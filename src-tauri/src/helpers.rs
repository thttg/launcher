use std::fs;
use std::path::Path;

use chardet::{charset2encoding, detect};
use charset_normalizer_rs::from_bytes;
use encoding::label::encoding_from_whatwg_label;
use encoding::DecoderTrap;

pub fn decode_buffer(buf: Vec<u8>) -> String {
    // Detect encoding using chardet
    let first_encoding = charset2encoding(&detect(&buf).0).to_string();

    // Detect encoding using charset_normalizer_rs
    let second_encoding = from_bytes(&buf, None)
        .get_best()
        .map_or_else(|| "not_found".to_string(), |cd| cd.encoding().to_string());

    // List of potential encodings to try
    let potential_encodings = [first_encoding.as_str(), second_encoding.as_str(), "UTF-8", "Windows-1251"];

    // Try decoding with each encoding and select the best result
    for &encoding_label in &potential_encodings {
        if let Some(encoding) = encoding_from_whatwg_label(encoding_label) {
            match encoding.decode(&buf, DecoderTrap::Replace) {
                Ok(decoded) => return decoded,
                Err(_) => continue,
            }
        }
    }

    // Return a default string or error message if all decoding attempts fail
    "Decoding failed".to_string()
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
                                        println!("ERROR: {}", e.to_string());
                                        return Err(e.to_string());
                                    }
                                }
                            }

                            match copy_files(entry.path(), dest.as_ref().join(entry.file_name())) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(e.to_string());
                                }
                            }
                        } else {
                            let copy_results =
                                fs::copy(entry.path(), dest.as_ref().join(entry.file_name()));
                            match copy_results {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("ERROR: {}", e.to_string());
                                    return Err(e.to_string());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("ERROR: {}", e.to_string());
                        return Err(e.to_string());
                    }
                }
            }
            return Ok(());
        }
        Err(e) => {
            println!("ERROR: {}", e.to_string());
            return Err(e.to_string());
        }
    }
}
