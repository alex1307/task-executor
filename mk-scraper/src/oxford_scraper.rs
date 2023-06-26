use common_libs::{utils::{trim_tabs, replace_tabs}, error::{ServiceExuctionResult, ServiceError, ServiceErrorType}};
use log::error;


use crate::{first_element, model::{DictionaryEntry, Dictionary}};

use super::{
    description, download_from_url, elements, merge_definitions, mp3_element, mp3_element_to_url,
    to_url,
};

const URL: &str = r#"https://www.oxfordlearnersdictionaries.com/definition/english/"#;
pub const MP3_QUERY: &str = r#"div[class="sound audio_play_button pron-uk icon-audio"]"#;
pub const MP3_START_WITH: &str = r#"https://"#;
pub const DEFINITION_QUERY: &str = r#"div[class="entry"] span[class="def"]"#;

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

pub fn check_for_more(word: &str, html: &str) -> Option<String> {
    let first = first_element(r#"link[rel="canonical"]"#, &html, false);

    if first.is_none() {
        return None;
    }

    let base_url = to_url(URL, word);

    let canonical = first.unwrap();
    let mut expected_url = String::from(r#"""#);
    expected_url.push_str(&base_url);
    expected_url.push_str(r#"""#);

    if canonical.contains(&expected_url) {
        return None;
    }

    for i in 1..4 {
        let mut search_url = String::from(&base_url);
        let id = vec![r#"_"#, &i.to_string()].join("");
        search_url.push_str(&id);

        if canonical.contains(&search_url) {
            let next_id = vec![r#"_"#, &(i + 1).to_string()].join("");
            search_url = search_url.replace(&id, &next_id);
            return Some(search_url);
        }
    }
    return None;
}

pub fn check_for_redirects(word: &str, html: &str) -> bool {
    let first = first_element(r#"link[rel="canonical"]"#, &html, false);

    if first.is_none() {
        return false;
    }

    let base_url = to_url(URL, word);
    let expected_url = vec![r#"""#, &base_url, r#"""#].join("");
    let canonical = first.unwrap();
    return !canonical.contains(&expected_url);
}

pub async fn scrape(word: &str) -> Option<DictionaryEntry> {
    if word.trim().is_empty() || word.trim().len() < 3 {
        error!("word.min.lenght.is.3");
        return None;
    }
    let mut trimmed = trim_tabs(&mut String::from(word));
    trimmed = replace_tabs(&mut trimmed, "-");
    let url = to_url(URL, &trimmed);
    let html = match download_from_url(&url).await {
        Ok(content) => content,
        Err(_) => return None,
    };

    if check_for_redirects(word, &html) {
        merge_definitions(
            vec![URL, &trimmed, r#"_1"#].join("").as_str(),
            vec![URL, &trimmed, r#"_2"#].join("").as_str(),
            vec![URL, &trimmed, r#"_3"#].join("").as_str(),
        )
        .await
    } else {
        let mp3_link = mp3_url(&html);
        let definitions = to_definitions(&html);

        if mp3_link.is_none() || definitions.is_empty() {
            return None;
        }

        Some(DictionaryEntry {
            source: Dictionary::Oxford,
            word: word.clone().to_lowercase(),
            url: to_url(URL, word),
            mp3_link,
            definitions,
            file: None,
        })
    }
}

pub async fn download(url: &str) -> ServiceExuctionResult<DictionaryEntry> {
    let html = (download_from_url(url).await)?;

    let mp3_link = mp3_url(&html);
    let definitions = to_definitions(&html);

    if mp3_link.is_none() || definitions.is_empty() {
        return Err(ServiceError {
            message: "definitions or/and mp3 files are not found".to_string(),
            error_type: ServiceErrorType::ResourceNotFound,
        });
    }

    let found = first_element(r#"div[class="webtop"] h1[class="headword"]"#, &html, true);
    let word = found.unwrap();
    Ok(DictionaryEntry {
        source: Dictionary::Oxford,
        word: word.clone(),
        url: to_url(URL, &word),
        mp3_link,
        definitions,
        file: None,
    })
}

#[cfg(test)]
mod oxford_unit_tests {
    use common_libs::files::{file_name, read_file_content};

    use crate::{oxford_scraper::{self, check_for_more}, merge_definitions, constants::HTML_EXT, download_from_url};

    
    #[tokio::test]
    async fn oxford_scraper_test() {
        let res = oxford_scraper::scrape(r#"correct"#).await;
        assert!(res.is_some());
        println!(
            "found: {}",
            serde_json::to_string_pretty(&res.unwrap()).unwrap()
        );
    }

    #[tokio::test]
    async fn scrape_wind_up_test() {
        let res = merge_definitions(
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/wind-up_1"#,
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/wind-up_2"#,
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/wind-up_3"#,
        )
        .await;
        assert!(res.is_some());
        println!("found: {:?}", res.clone().unwrap());

        let scrape = oxford_scraper::scrape("wind-up").await;
        assert!(scrape.is_some());
        println!("found: {:?}", scrape.clone().unwrap());

        assert_eq!(
            res.unwrap().definitions.len(),
            scrape.unwrap().definitions.len()
        );
    }

    #[tokio::test]
    async fn to_next_word3_test() {
        let word = "correct_2";
        let file_name = file_name("download/tmp/oxford/", word, HTML_EXT);
        let html = match read_file_content(&file_name) {
            Ok(cnt) => {
                println!("size: {}", cnt.len());
                String::from_utf8(cnt).unwrap()
            }
            _ => String::new(),
        };
        assert_eq!(
            Some(String::from(
                r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_3"#
            )),
            check_for_more("correct", &html)
        );

        assert!(
            download_from_url(
                r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_3"#
            )
            .await.is_err()
        );

        assert!(download_from_url(
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_2"#
        )
        .await
        .is_ok());
        assert!(download_from_url(
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_1"#
        )
        .await
        .is_ok());
        assert!(download_from_url(
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct"#
        )
        .await
        .is_ok());

        let correct = oxford_scraper::scrape(r#"correct"#).await;
        assert!(correct.is_some());
        println!("found: {:?}", correct.unwrap());

        let correct = oxford_scraper::download(
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_1"#,
        )
        .await;
        assert!(correct.is_ok());
        println!("found: {:?}", correct.unwrap());

        let correct = oxford_scraper::download(
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_2"#,
        )
        .await;
        assert!(correct.is_ok());
        println!("found: {:?}", correct.unwrap());
    }
}
