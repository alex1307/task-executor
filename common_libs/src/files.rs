use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::Path;

use log::info;
use serde::{de::DeserializeOwned, Serialize};

use crate::MAX_32_MB;
use crate::error::{DictionaryError, DictionaryErrorKind, ServiceExuctionResult, ServiceError, ServiceErrorType};
use crate::utils::{trim_tabs, from_binary};

const SPLITTERS: [&str; 8] = [",", ";", ":", "?", "!", "/", "\\", "."];
const FILES_NOT_FOUND: &str = "files.not.found";

pub fn find_file<T: AsRef<str>>(dir: T, search_pattern: T) -> Vec<String> {
    let output = std::process::Command::new("find")
        .arg(dir.as_ref())
        .arg("-type")
        .arg("file")
        .arg("-name")
        .arg(search_pattern.as_ref())
        .output()
        .expect(FILES_NOT_FOUND);
        let response = String::from_utf8_lossy(&output.stdout);
        let lines = response.lines();
        let mut names = vec![];
        for l in lines.into_iter() {
            names.push(l.to_lowercase());
        }
        names
}

pub fn file_name<T: AsRef<str>>(folder: T, name: T, ext: &str) -> String {
    let mut file_name = String::from(folder.as_ref());
    if !file_name.ends_with("/") {
        file_name.push_str("/")
    }
    file_name.push_str(name.as_ref());
    if !file_name.ends_with(ext) {
        file_name.push_str(ext);
    }
    file_name
}

pub fn list_all_file_names<T: AsRef<str>>(dir_name: T) -> Result<Vec<String>, DictionaryError> {
    if !{
        let ref this = Path::new(dir_name.as_ref());
        fs::metadata(this).is_ok()
    } {
        return Err(DictionaryError::throw("dir.not.found", DictionaryErrorKind::NotFound));
    }

    let paths = fs::read_dir(dir_name.as_ref()).unwrap();
    let mut files = Vec::new();

    for entry in paths {
        let dir_entry = entry?;
        let path = dir_entry.path();
        if !path.is_dir() {
            let file_name = if path.file_name().is_some() {
                path.file_name().unwrap().to_str().unwrap()
            } else {
                return Err(DictionaryError::throw("invalid.file", DictionaryErrorKind::NotFound));
            };
            files.push(String::from(file_name));
        }
    }

    Ok(files)
}

pub fn list_all_files<T: AsRef<str>>(source: T) -> Result<Vec<String>, DictionaryError> {
    let dir_name =source.as_ref();
    if !Path::new(dir_name).exists() {
        return Err(DictionaryError::throw("dir.not.found", DictionaryErrorKind::NotFound));
    }

    let paths = fs::read_dir(dir_name).unwrap();
    let mut files = Vec::new();

    for entry in paths {
        let dir_entry = entry?;
        let path = dir_entry.path();
        if !path.is_dir() {
            let file_name = if path.file_name().is_some() {
                path.file_name().unwrap().to_str().unwrap()
            } else {
                return Err(DictionaryError::throw("invalid.file", DictionaryErrorKind::NotFound));
            };
            let mut full_file_name = String::from(dir_name);
            if!dir_name.ends_with("/") {
                full_file_name.push_str("/");
            }
            full_file_name.push_str(file_name);
            files.push(String::from(full_file_name));
        }
    }

    Ok(files)
}

pub fn read_file_content<T: AsRef<str>>(file_name: T) -> ServiceExuctionResult<Vec<u8>> {
    let mut reader: Box<dyn Read> = if !file_name.as_ref().is_empty() || !Path::new(file_name.as_ref()).exists() {
        Box::new(BufReader::new(File::open(file_name.as_ref())?))
    } else {
        return Err(ServiceError{
            message: "file.not.found".to_string(),
            error_type: ServiceErrorType::ResourceNotFound
        });
    };
    let mut buffer = [0; MAX_32_MB];
    let num_read = reader.read(&mut buffer)?;
    if num_read == 0 {
        Err(ServiceError{
            message: "invalid.content".to_string(),
            error_type: ServiceErrorType::ResourceNotFound
        })
    } else {
        Ok(Vec::from(&buffer[..num_read]))
    }
}

pub fn save<T>(file_name: &str, entity: &T) -> Result<(), DictionaryError>
where
    T: ?Sized + Serialize,
{
    if !file_name.is_empty() {
        let writer = Box::new(BufWriter::new(File::create(file_name)?));
        serde_json::to_writer_pretty(writer, entity)?;
    } else {
        info!("{}", serde_json::to_string_pretty(entity)?);
    }
    Ok(())
}

pub fn from_file<S: AsRef<str> ,T>(file_name: S) -> Option<T>
where
    T: ?Sized + DeserializeOwned,
{
    let try_to_read_file = read_file_content(file_name.as_ref());
    let binary = if try_to_read_file.is_ok() {
        try_to_read_file.unwrap()
    } else {
        return None;
    };

    match  from_binary::<T>(binary){
        Ok(t) => Some(t),
        Err(_) => None,
    }
}

pub fn parse_file<T: AsRef<str>>(file_name: T) -> ServiceExuctionResult<Vec<String>> {
    let binary = read_file_content(file_name)?;
    let mut content = String::from_utf8(binary)?;
    Ok(split(&mut content))
}

fn split<T: AsMut<str>>(source: &mut T) -> Vec<String> {
    let result = source.as_mut();
    if result.trim().is_empty() {
        return vec![];
    }
    
    let mut result = trim_tabs(result.trim());

    for ch in SPLITTERS.iter() {
        result = result
                        .replace(ch, "\n")
                        .trim()
                        .to_string();        
    }
    let lines: Vec<String> = result.lines().map(|s| String::from(s)).collect();
    lines
}

#[cfg(test)]
pub mod file_tests {
    use crate::files::{file_name, list_all_file_names, parse_file, read_file_content, split};
    use std::{path::Path, vec};

    use super::find_file;

    #[test]
    fn list_files_test() {
        let result = list_all_file_names("not/found/");
        assert!(result.is_err());

        let result = list_all_file_names("test/en-en/");
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(12, files.len());
    }

    #[test]
    fn read_file_content_test() {
        let result = read_file_content("");
        assert!(result.is_err());

        let result = read_file_content("test/en-en/file_does_not_exist.txt");
        assert!(result.is_err());

        let result = read_file_content("test/en-en/cease.json");
        assert!(result.is_ok());

        let binary = result.unwrap();
        assert_eq!(1281, binary.len());
    }

    #[test]
    fn parse_text_test() {
        let mut  text = String::from("do: something, but nothing\n lets; play, what/ ever! do it? not do it\n dfadsfa\n end. A.");
        let result = split(&mut text);
        assert!(!result.is_empty());
        assert_eq!(12, result.len());
        for any in result {
            println!("-> {}", any);
        }
    }

    #[test]
    fn parse_file_test() {
        let words = parse_file("test/download.txt").unwrap();
        assert!(!words.is_empty());
        assert_eq!(12, words.len());
    }

    #[test]
    fn is_downloaded_test() {
        let json_file = file_name("test/", "download", ".txt");
        let mp3_file = file_name("test/mp3", "cease", ".mp3");
        assert!(Path::new(&json_file).exists() && Path::new(&mp3_file).exists())
    }

    #[test]
    fn find_files_test() {
        let found = find_file("test", "*.json");
        assert!(!found.is_empty());
        assert!(found.len() >= 1);

        let found = find_file("test", "*.mp3");
        assert!(!found.is_empty());
        assert!(found.len() >= 1);
    }

    #[test]
    fn files_not_found_test() {
        let not_found = find_file(String::from("test"), String::from("*.xyz"));
        let empty: Vec<String> = vec![];
        assert_eq!(empty, not_found);

        let not_found = find_file("xyz/ttt", "*.mp3");
        assert_eq!(empty, not_found);
    }
}
