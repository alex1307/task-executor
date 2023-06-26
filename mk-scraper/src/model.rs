use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Dictionary {
    Collins,
    Cambridge,
    Oxford,
    Undefined,
}

impl From<String> for Dictionary {
    fn from(source: String) -> Self {
        match source.to_lowercase().trim() {
            "cambridge" => Dictionary::Cambridge,
            "collins" => Dictionary::Collins,
            "oxford" => Dictionary::Oxford,
            _ => Dictionary::Undefined,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DictionaryEntry {
    pub source: Dictionary,
    pub url: String,
    pub word: String,
    pub mp3_link: Option<String>,
    pub file: Option<String>,
    pub definitions: Vec<String>,
}