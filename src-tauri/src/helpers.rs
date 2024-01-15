use std::fs;
use std::path::Path;

use chardet::{charset2encoding, detect};
use charset_normalizer_rs::from_bytes;
use encoding::label::encoding_from_whatwg_label;
use encoding::DecoderTrap;
use log::info;

/// Decode a buffer and return the output string along with the detected encodings.
pub fn decode_buffer(buf: Vec<u8>) -> (String, String, String) {
    let detected_encoding = detect_encoding(&buf);
    let output = decode_using_detected_encoding(&buf, &detected_encoding);

    (output, detected_encoding.0, detected_encoding.1)
}

/// Detects the encoding of the given buffer using `chardet` and `charset_normalizer_rs`.
fn detect_encoding(buf: &[u8]) -> (String, String) {
    let first_encoding = charset2encoding(&detect(buf).0).to_string();
    let second_encoding = from_bytes(buf, None)
        .get_best()
        .map_or("not_found".to_string(), |cd| cd.encoding().to_string());

    (first_encoding, second_encoding)
}

/// Decodes the given buffer using the specified encoding.
fn decode_using_detected_encoding(buf: &[u8], encoding: &(String, String)) -> String {
    let str_encoding = select_appropriate_encoding(&encoding.0, &encoding.1);

    encoding_from_whatwg_label(&str_encoding)
        .map_or_else(
            || String::from_utf8_lossy(buf).to_string(),
            |coder| coder.decode(buf, DecoderTrap::Ignore).unwrap_or_default(),
        )
}

/// Selects the appropriate encoding based on the detected encodings.
fn select_appropriate_encoding(first: &str, second: &str) -> String {
    let mut str_encoding = first.to_string();

    if ["KOI8-R", "MacCyrillic", "x-mac-cyrillic", "koi8-r", "macintosh", "ibm866"].contains(&first) {
        str_encoding = "cp1251".to_string();
    }

    str_encoding
}

/// Copies files from the source path to the destination path.
pub fn copy_files(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<(), String> {
    let files = fs::read_dir(src).map_err(|e| e.to_string())?;

    for entry in files {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;

        if ty.is_dir() {
            handle_directory_entry(&entry, &dest)?;
        } else {
            handle_file_entry(&entry, &dest)?;
        }
    }

    Ok(())
}

/// Handles the directory entry while copying files.
fn handle_directory_entry(entry: &fs::DirEntry, dest: &Path) -> Result<(), String> {
    let dir_path = dest.join(entry.file_name());
    if fs::create_dir(&dir_path).is_err() {
        println!("Directory {:?} already exists or cannot be created", dir_path);
    }
    copy_files(entry.path(), dir_path)
}

/// Handles the file entry while copying files.
fn handle_file_entry(entry: &fs::DirEntry, dest: &Path) -> Result<(), String> {
    fs::copy(entry.path(), dest.join(entry.file_name()))
        .map(|_| ())
        .map_err(|e| e.to_string())
}
