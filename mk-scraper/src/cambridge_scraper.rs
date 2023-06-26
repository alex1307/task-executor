use common_libs::files::file_name;
use log::error;



use crate::constants::HTML_EXT;

use super::{
    description, download_from_url, elements, first_element, mp3_element, mp3_element_to_url,
    to_url, wget, DictionaryEntry,
};
const BASE_URL: &str = r#"https://dictionary.cambridge.org"#;
const URL: &str = r#"https://dictionary.cambridge.org/dictionary/english/"#;
pub const MP3_QUERY: &str = r#"source[type="audio/mpeg"]"#;
pub const MP3_START_WITH: &str = r#"/media"#;
pub const DEFINITION_QUERY: &str = r#"div[class="def ddef_d db"]"#;

fn mp3_url(html: &str) -> Option<String> {
    let element = mp3_element(MP3_QUERY, &html);

    if element.is_none() {
        return None;
    }

    let url = mp3_element_to_url(&element.unwrap(), MP3_START_WITH);

    if url.is_none() {
        return None;
    }

    let mut mp3_url = String::from(BASE_URL);
    mp3_url.push_str(&url.unwrap());
    Some(mp3_url)
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
            let file_name = file_name("download/tmp/cambridge", word, HTML_EXT);
            match wget(&url, &file_name) {
                Ok(html) => html,
                Err(_err) => {
                    error!("resource.not.found: {}", url);
                    return None;
                }
            }
        }
    };

    let mp3_link = mp3_url(&html);
    let definitions = to_definitions(&html);

    if mp3_link.is_none() || definitions.is_empty() {
        return None;
    }

    let mut found = first_element(
        r#"div[class="di-title"] h2[class="headword tw-bw dhw dpos-h_hw "] b"#,
        &html,
        true,
    );

    if found.is_none() {
        found = first_element(
            r#"div[class="di-title"] h2[class="headword tw-bw dhw dpos-h_hw "] span"#,
            &html,
            true,
        );
    }


    let word = match found {
        Some(v) => v,
        None => word.to_lowercase()
    };

    Some(DictionaryEntry {
        source: super::Dictionary::Cambridge,
        word: word.clone().to_lowercase(),
        url: to_url(URL, &word),
        mp3_link,
        definitions,
        file: None,
    })
}

#[cfg(test)]
mod cambridge_unit_tests {
    use crate::cambridge_scraper;

    
    #[tokio::test]
    async fn cambridge_scraper_not_found_test() {
        let res = cambridge_scraper::scrape(r#"amisega"#).await;
        assert!(res.is_none());


        let res = cambridge_scraper::scrape(r#"undertaken"#).await;
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn cambridge_fail_test() {
        let res = cambridge_scraper::scrape(r#"undertake"#).await;
        assert!(res.is_some());
        println!(
            "found: {}",
            serde_json::to_string_pretty(&res.unwrap()).unwrap()
        );
    }

    #[tokio::test]
    async fn cambridge_scraper_test() {
        let words = vec![r#"wind up"#, 
            r#"bail out"#,
            r#"endorsement"#];
        for w in words {
            let res = cambridge_scraper::scrape(w).await;
            assert!(res.is_some());
            println!(
                "found: {}",
                serde_json::to_string_pretty(&res.unwrap()).unwrap()
            );
        }    
        
    }
}
