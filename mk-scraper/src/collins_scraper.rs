use common_libs::files::file_name;
use log::error;
use crate::{model::DictionaryEntry, constants::HTML_EXT};

use super::{
    description, elements, mp3_element, mp3_element_to_url, to_url, wget, download_from_url,
};

const URL: &str = r#"https://www.collinsdictionary.com/dictionary/english/"#;
pub const MP3_QUERY: &str =
    r#"a[class="hwd_sound sound audio_play_button icon-volume-up ptr"]"#;
pub const MP3_START_WITH: &str = r#"https:"#;
pub const DEFINITION_QUERY: &str = r#"div[class="sense"] div[class="def"]"#;

fn mp3_url(html: &str) -> Option<String> {
    let element = mp3_element(MP3_QUERY, &html);

    if element.is_none() {
        return None;
    }

    mp3_element_to_url(&element.unwrap(), MP3_START_WITH)
}

fn to_definitions(html: &str) -> Vec<String> {
    let definitions = elements(DEFINITION_QUERY, &html, true);
    definitions
        .into_iter()
        .map(|e| description(e))
        .filter(Option::is_some)
        .map(Option::unwrap)
        .collect()
}

pub async fn scrape(word: &str) -> Option<DictionaryEntry> {
    if word.trim().is_empty() || word.trim().len() < 3 {
        error!("word.min.lenght.is.3");
        return None;
    }

    let url = to_url(URL, word);
    let html = match download_from_url(&url).await {
        Ok(content) => content,
        Err(_err) => {
            error!("failed.to.read.from.url: {}", url);
            let file_name = file_name("download/tmp/collins", word, HTML_EXT);
            match wget(&url, &file_name) {
                Ok(html) => html,
                Err(_err) => {
                    error!("resource.not.found: {}", url);
                    return None
                }
            }
        }
    };
    let mp3_link = mp3_url(&html);
    let definitions = to_definitions(&html);

    Some(DictionaryEntry {
        source: super::Dictionary::Cambridge,
        word: word.clone().to_lowercase(),
        url: to_url(URL, word),
        mp3_link,
        definitions,
        file: None,
    })
}
#[cfg(test)]
mod collins_unit_tests{
    use crate::collins_scraper;

    
    #[tokio::test]
    async fn scrape_test() {
        let html = collins_scraper::scrape("dust up").await;
        assert!(html.is_some());
        println!("{:?}", html.unwrap());
    }

    #[tokio::test]
    async fn collins_scraper_test() {
        let res = collins_scraper::scrape(r#"correct"#).await;
        assert!(res.is_some());
        println!(
            "found: {}",
            serde_json::to_string_pretty(&res.unwrap()).unwrap()
        );
    }

}
