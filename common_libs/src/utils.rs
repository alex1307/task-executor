use log::error;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::de::DeserializeOwned;
use std::iter::FromIterator;

use crate::error::{RuntimeResult, RuntimeError};

const DOUBLE_TABS: &str = "  ";
const SINGLE_TAB: &str = " ";

pub fn random_in_range(n: usize) -> Vec<usize> {
    let mut vec: Vec<usize> = (0..n).collect();
    vec.shuffle(&mut thread_rng());
    println!("{:?}", vec);
    vec
}

pub fn shuffle<T: Clone>(source: Box<Vec<T>>) -> Vec<T> {
    let mut rng = rand::thread_rng();
    let mut shuffled = Vec::from_iter(source.iter().cloned());
    shuffled.shuffle(&mut rng);
    shuffled
}

pub fn shuffle_descriptions(source: &mut Vec<String>) -> Vec<String> {
    let size = source.len();
    let shuffled_idx = random_in_range(size);
    let mut shuffled_descriptions = Vec::new();
    for idx in shuffled_idx.into_iter() {
        match source.get(idx) {
            None => continue,
            Some(desc) => {
                shuffled_descriptions.push(desc.clone());
            }
        }
    }
    shuffled_descriptions
}

pub fn trim_tabs <T: AsRef<str>> (source: T) -> String {
    let mut trimmed = String::from(source.as_ref());
    while trimmed.contains(DOUBLE_TABS) {
        trimmed =  trimmed.replace(DOUBLE_TABS, SINGLE_TAB);
    }
    trimmed.trim().to_string()
}

pub fn replace_tabs<T: AsRef<str>>(source: T, replacement: &str) -> String {
    let mut trimmed = trim_tabs(source);
    trimmed = trimmed.to_lowercase();
    trimmed = trimmed.replace(SINGLE_TAB, replacement.clone());
    trimmed
}

pub fn from_binary<T>(binary: Vec<u8>) -> RuntimeResult<T>
    where T: ?Sized + DeserializeOwned,
{
    
    if binary.is_empty() {
        error!("binary.empty");
        return Err(RuntimeError{
            message: "binary.empty".to_string(),
            error_type: crate::error::RuntimeErrorType::InvalidData
        });
    }
    
    let string = match String::from_utf8(binary) {
        Ok(utf8) => utf8,
        Err(err) => {
            error!("failed.parsing.binary => Err: {}", err);
            return Err(RuntimeError{
                message: "invalid.binary".to_string(),
                error_type: crate::error::RuntimeErrorType::InvalidData
            });
        }
    };
    Ok(serde_json::from_str::<T>(&string)?)
}