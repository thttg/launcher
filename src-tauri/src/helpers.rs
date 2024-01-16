use std::fs;
use std::path::Path;

use encoding_rs::*;
use chardetng::EncodingDetector;
use log::info;

// List of common encodings to try if automatic detection fails.
static COMMON_ENCODINGS: &[&'static Encoding] = &[
    WINDOWS_1252, // Common for Western European languages
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

/// Decodes a buffer of bytes into a string using the best guess for encoding.
/// Returns the decoded string, the name of the detected encoding, and the name of the used encoding.
pub fn decode_buffer(buf: Vec<u8>) -> (String, String, String) {
    let mut detector = EncodingDetector::new(); // Create a new detector instance
    detector.feed(&buf, true); // Feed the buffer to the detector
    let detected_encoding = detector.guess(None, true); // Guess the encoding

    // Determine the preferred encoding for decoding
    let preferred_encoding = Encoding::for_label(detected_encoding.name().as_bytes())
        .or_else(|| try_common_encodings(&buf))
        .unwrap_or(UTF_8);

    // Decode the buffer using the preferred encoding
    let (decoded_text, _, _) = preferred_encoding.decode(&buf);
    // Return the decoded text along with the detected and used encoding names
    (decoded_text.into_owned(), detected_encoding.name().to_string(), preferred_encoding.name().to_string())
}

/// Tries to decode the buffer using a list of common encodings.
/// Returns the first encoding that successfully decodes the buffer without errors.
fn try_common_encodings(buf: &[u8]) -> Option<&'static Encoding> {
    for &encoding in COMMON_ENCODINGS {
        let (decoded, _, had_errors) = encoding.decode(buf);
        if !had_errors {
            return Some(encoding); // Return the first successful encoding
        }
    }
    None // Return None if no encoding succeeded
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
