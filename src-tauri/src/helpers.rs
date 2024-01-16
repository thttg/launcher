use std::fs;
use std::path::Path;

use chardet::{charset2encoding, detect};
use charset_normalizer_rs::from_bytes;
use encoding::label::encoding_from_whatwg_label;
use encoding::DecoderTrap;
use log::info;

static COMMON_ENCODINGS: &[&Encoding] = &[
    UTF_8,
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

pub fn decode_buffer(buf: Vec<u8>) -> (String, String, String) {
    let first_encoding = charset2encoding(&detect(&buf).0).to_string();
    let second_encoding = from_bytes(&buf, None)
        .get_best()
        .map_or_else(|| "not_found".to_string(), |cd| cd.encoding().to_string());

    let mut str_encoding = if first_encoding == second_encoding {
        first_encoding
    } else {
        try_common_encodings(&buf).unwrap_or_else(|| first_encoding)
    };

    let decoded_text = match encoding_from_whatwg_label(&str_encoding) {
        Some(coder) => coder.decode(&buf, DecoderTrap::Replace).unwrap_or_else(|_| "Error".to_string()),
        None => String::from_utf8_lossy(&buf).to_string(),
    };

    (decoded_text, first_encoding, second_encoding)
}

fn try_common_encodings(buf: &[u8]) -> Option<String> {
    for &encoding in COMMON_ENCODINGS {
        if let Ok(decoded) = encoding.decode(buf, DecoderTrap::Replace) {
            if !decoded.contains('\u{FFFD}') {
                return Some(encoding.name().to_string());
            }
        }
    }
    None
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
