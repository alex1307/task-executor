pub mod cambridge_scraper;
pub mod collins_scraper;
pub mod oxford_scraper;
pub mod unit_tests;
pub mod constants;
pub mod model;
pub mod task_executor;

use std::io::Cursor;


use common_libs::{error::{ServiceExuctionResult, ServiceError, ServiceErrorType, DictionaryError, DictionaryErrorKind}, files::read_file_content, utils::{trim_tabs, replace_tabs}};
use constants::{USER_AGENT, INVALID, MP3_EXT};
use itertools::Itertools;
use log::{debug, error, info};
use model::{DictionaryEntry, Dictionary};
use reqwest::StatusCode;
use scraper::{Html, Selector};

use futures::future::join_all;
use tokio::runtime;
use crate::constants::NOT_FOUND;


pub fn wget(url: &str, file_name: &str) -> ServiceExuctionResult<String> {
    let status = std::process::Command::new("wget")
    .arg("--user-agent")
    .arg(USER_AGENT)
    .arg(url)
    .arg("-O")
    .arg(file_name)
    .status();

    if status.is_err() {
        error!("failed.executing.wget: {}", status.err().unwrap());
        return Err(ServiceError {
            message: "wget.failed".to_string(),
            error_type: ServiceErrorType::Failure,
        });
    }

    let bytes = read_file_content(file_name)?;
    Ok(String::from_utf8(bytes)?)
}

pub async fn fetch_url(url: String, file_name: String) -> ServiceExuctionResult<()> {
    debug!("downloading from [{}] to {}", url, file_name);
    let response = reqwest::get(url.clone()).await.unwrap();
    let status = response.status();
    info!("download status: {}", status);
    if status.is_success() {
        let mut file = std::fs::File::create(file_name.clone())?;
        let mut content = Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;
        Ok(())
    } else {
        match wget(&url, &file_name) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }
    
}

async fn download_from_url(url: &str) -> ServiceExuctionResult<String> {
    println!("URL: {}", url.clone());
    let response = reqwest::get(url).await?;
    println!("resposne: {:?}", response);
    if StatusCode::OK != response.status() {
        let msg = format!("nok.status.code: {}", response.status().as_str());
        return Err(ServiceError {
            message: msg,
            error_type: ServiceErrorType::Unavailable,
        });
    }

    let bytes = response.bytes().await.unwrap();
    let slice = &bytes[..];
    let content = String::from_utf8(Vec::from(slice)).unwrap();
    let lower = content.to_lowercase();
    for i in 0..NOT_FOUND.len() {
        if lower.contains(NOT_FOUND[i]) {
            return Err(ServiceError {
                message: "not.found".to_string(),
                error_type: ServiceErrorType::ResourceNotFound,
            });
        }
    }
    Ok(content)
}

fn description(inner_html: String) -> Option<String> {
    let mut replaced = match inner_text(inner_html.clone(), "a") {
        Ok(inner) => inner,
        Err(_err) => return None,
    };

    replaced = match inner_text(replaced, "span") {
        Ok(inner) => inner,
        Err(_) => return None,
    };

    replaced = replaced.replace("\n", " ");
    replaced = trim_tabs(&mut replaced);
    replaced = replaced.replace(" .", ".");
    Some(replaced)
}

fn inner_text(inner_html: String, tag: &str) -> Result<String, DictionaryError> {
    let mut replaced = inner_html.clone();
    let mut open_tag = String::from("<");
    open_tag.push_str(tag);
    let mut close_tag = String::from("</");
    close_tag.push_str(tag);
    close_tag.push_str(">");

    if !(inner_html.contains(&open_tag) && inner_html.contains(&close_tag)) {
        return Ok(inner_html);
    }

    let tag_selector = Selector::parse(tag).unwrap();

    while replaced.contains(&open_tag) && replaced.contains(&close_tag) {
        let start = replaced.find(&open_tag).unwrap();
        let end = replaced.find(&close_tag).unwrap();
        if start < end + close_tag.len() {
            let to_be_replaced = &replaced[start..end + close_tag.len()];
            let a_href = Html::parse_fragment(&replaced);
            let inner = a_href.select(&tag_selector).next().unwrap().inner_html();
            replaced = replaced.replace(to_be_replaced, &inner);
        } else {
            error!(
                "parse.error: start postion {} is bofore end: {}",
                start,
                end + close_tag.len()
            );
            return Err(DictionaryError::throw(
                "unable.to.parse.definition",
                DictionaryErrorKind::InvalidData,
            ));
        }
    }
    Ok(replaced)
}

fn elements(query: &str, html_content: &str, inner: bool) -> Vec<String> {
    let fragment = Html::parse_fragment(&html_content);
    let selector = Selector::parse(query).unwrap();
    let elements = fragment.select(&selector).into_iter();
    let mut all = Vec::new();
    for e in elements {
        if inner {
            all.push(e.inner_html());
        } else {
            all.push(e.html());
        }
    }
    all
}

fn first_element(query: &str, html_content: &str, inner: bool) -> Option<String> {
    let fragment = Html::parse_fragment(&html_content);
    let selector = Selector::parse(query).unwrap();
    let mut elements = fragment.select(&selector).into_iter();
    let found = elements.next();

    if found.is_some() {
        return if inner {
            Some(found.unwrap().inner_html())
        } else {
            Some(found.unwrap().html())
        };
    }

    None
}

fn mp3_element(mp3_query: &str, html_content: &str) -> Option<String> {
    first_element(mp3_query, html_content, false)
}

pub async fn merge_definitions(url1: &str, url2: &str, url3: &str) -> Option<DictionaryEntry> {
    let source_1 = oxford_scraper::download(url1).await;
    let source_2 = oxford_scraper::download(url2).await;

    if source_1.is_err() {
        return None;
    }

    if source_2.is_err() {
        return None;
    }

    let mut merged = source_1.unwrap();
    for def in source_2.unwrap().definitions {
        merged.definitions.push(def);
    }
    if !url3.is_empty() {
        let source_3 = oxford_scraper::download(url3).await;
        if source_3.is_ok() {
            for def in source_3.unwrap().definitions {
                merged.definitions.push(def);
            }
        }
    }

    Some(merged)
}

fn to_url(base_url: &str, word: &str) -> String {
    if word.is_empty() {
        return INVALID.to_lowercase();
    }

    if base_url.is_empty() {
        return INVALID.to_lowercase();
    }
    let mut w = String::from(word);
    let mut trimmed = trim_tabs(&mut w);
    let processed = replace_tabs(&mut trimmed, "-");
    let mut url = String::from(base_url);
    url.push_str(&processed);
    url.to_lowercase()
}

fn mp3_element_to_url(source: &str, start_with: &str) -> Option<String> {
    element_to_url(source, start_with, MP3_EXT)
}

fn element_to_url(source: &str, start_with: &str, end_with: &str) -> Option<String> {
    let values: Vec<&str> = source.split_whitespace().collect();
    let found = values
        .into_iter()
        .find_or_first(|s| s.contains(start_with) && s.contains(end_with));

    let raw_element = if found.is_some() {
        found.unwrap()
    } else {
        return None;
    };

    let start = raw_element.find(start_with).unwrap();
    let end = raw_element.find(end_with).unwrap() + end_with.len();
    Some(raw_element[start..end].to_lowercase())
}

pub async fn scrape_it_from<S: AsRef<str>>(dictionary: String, word: S) -> Option<DictionaryEntry> {
    info!("scrape from {}", dictionary);
    match Dictionary::from(dictionary) {
        Dictionary::Cambridge => cambridge_scraper::scrape(word.as_ref()).await,
        Dictionary::Collins => collins_scraper::scrape(word.as_ref()).await,
        Dictionary::Oxford => oxford_scraper::scrape(word.as_ref()).await,
        _ => return None,
    }
}

pub  async fn scrape_all(source: Vec<String>) {
    info!("scrape_all is being executed for {:?}", source);
    let mut tasks = vec![];
    for w in source {
        let f1 = scrape_it_from("Cambridge".to_lowercase(), w.clone());
        let f2 = scrape_it_from("Collins".to_lowercase(), w.clone());
        let f3 = scrape_it_from("Oxford".to_lowercase(), w.clone());
        tasks.push(f1);
        tasks.push(f2);
        tasks.push(f3);
    }    
    let _ = join_all(tasks);    
}

pub  async fn tokio_scrape_all(source: Vec<String>) {
    info!("scrape_all is being executed for {:?}", source);
    // let mut tasks = vec![];
    let runtime = runtime::Runtime::new().unwrap();
    
    for w in source {
        let nw = w.clone();
        runtime.block_on(async move {
            tokio::spawn(scrape_it_from("Cambridge".to_lowercase(), nw.clone())).await.unwrap();
        });
        let nw = w.clone();
        runtime.block_on(async move {
            tokio::spawn(scrape_it_from("Collins".to_lowercase(), nw.clone())).await.unwrap();
        });

        runtime.block_on(async move {
            tokio::spawn(scrape_it_from("Oxford".to_lowercase(), w.clone())).await.unwrap();
        });
        // let _f1 = tokio::spawn(scrape_it_from("Cambridge".to_lowercase(), w.clone())).await;
        // let _f2 = tokio::spawn(scrape_it_from("Cambridge".to_lowercase(), w.clone())).await;
        // let _f3 = tokio::spawn(scrape_it_from("Cambridge".to_lowercase(), w.clone())).await;
        // let f1 = scrape_it_from("Cambridge".to_lowercase(), w.clone());
        // let f2 = scrape_it_from("Collins".to_lowercase(), w.clone());
        // let f3 = scrape_it_from("Oxford".to_lowercase(), w.clone());
        // tasks.push(f1);
        // tasks.push(f2);
        // tasks.push(f3);
    }    
    // let _ = join_all(tasks);    
}