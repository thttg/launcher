use std::fs;
use std::path::Path;
use std::borrow::Cow;

use chardet::{charset2encoding, detect};
use charset_normalizer_rs::from_bytes;
use encoding_rs::*;
use log::info;

// Define a constant for possible encodings
const POSSIBLE_ENCODINGS: &[&'static Encoding] = &[
    UTF_8, WINDOWS_1252, // Common for Western European languages
    WINDOWS_1251, // Common for Cyrillic scripts
    WINDOWS_1256, // Arabic
    WINDOWS_1253, // Greek
    WINDOWS_1254, // Turkish
    WINDOWS_1257, // Baltic languages
    WINDOWS_1250, // Central European languages
    GB18030, BIG5, // Chinese
    EUC_KR, // Korean
    SHIFT_JIS, // Japanese
    WINDOWS_1258, // Vietnamese
    WINDOWS_874, // Thai
];

pub fn decode_buffer(buf: Vec<u8>) -> (String, String, String) {
    // Attempt to detect encoding using chardet
    let chardet_result = detect(&buf);
    let first_encoding = charset2encoding(&chardet_result.0).to_string();

    // If chardet has high confidence, use its result
    if chardet_result.1 > 0.9 {
        if let Some(encoding) = Encoding::for_label(first_encoding.as_bytes()) {
            let (decoded, _, _) = encoding.decode(&buf);
            return (decoded.into_owned(), first_encoding, "not_used".to_string());
        }
    }

    // If chardet confidence is low, use charset_normalizer_rs for detection
    let second_encoding = match from_bytes(&buf, None).get_best() {
        Some(cd) => cd.encoding().to_string(),
        None => "not_found".to_string(),
    };

    // Try to decode using possible encodings
    let buff_output = POSSIBLE_ENCODINGS.iter()
        .filter_map(|&enc| {
            let (decoded, _, _) = enc.decode(&buf);
            Some(decoded.into_owned())
        })
        .next()
        .unwrap_or_else(|| String::from_utf8_lossy(&buf).to_string());

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
