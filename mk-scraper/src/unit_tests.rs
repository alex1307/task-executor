#[cfg(test)]
mod runit {
    use common_libs::files::{file_name, read_file_content};
    use itertools::Itertools;

    use crate::{to_url, download_from_url, mp3_element, first_element, description, elements, constants::{PAGE_NOT_FOUND, NOT_FOUND, HTML_EXT}, merge_definitions, oxford_scraper::check_for_more};


    const CAMBRIDGE_URL: &str = "https://dictionary.cambridge.org/dictionary/english/";
    const OXFORD_URL: &str = r#"https://www.oxfordlearnersdictionaries.com/definition/english/"#;

    #[test]
    fn url_test() {
        assert_eq!(
            "https://dictionary.cambridge.org/dictionary/english/dust-up",
            to_url(CAMBRIDGE_URL, "dust up")
        );

        assert_eq!(
            "https://www.oxfordlearnersdictionaries.com/definition/english/dust-up",
            to_url(OXFORD_URL, "dust up")
        );
    }

    #[tokio::test]
    async fn not_found_test() {
        let words = vec!["amisega", "tazi e niama", "гьон сурат", "villian"];
        for w in words {
            let url = to_url(CAMBRIDGE_URL, w);
            assert!(download_from_url(&url).await.is_err());
            let url = to_url(OXFORD_URL, w);
            assert!(download_from_url(&url).await.is_err());
        }
    }

    #[tokio::test]
    async fn scrape_words_success_test() {
        let words = vec!["dust up", "cling", "stark", "villain", "bail out"];
        for w in words {
            let url = to_url(CAMBRIDGE_URL, w);
            assert!(download_from_url(&url).await.is_ok());
            let url = to_url(OXFORD_URL, w);
            assert!(download_from_url(&url).await.is_ok());
        }
    }

    #[tokio::test]
    async fn mp3_url_test() {
        let url = to_url(OXFORD_URL, "bail out");
        let found = download_from_url(&url).await;
        assert!(found.is_ok());
        let html = found.unwrap();
        let mp3 = mp3_element(
            r#"div[class="sound audio_play_button pron-uk icon-audio"]"#,
            &html,
        )
        .unwrap();
        println!("mp3 element: {}", mp3);
        assert!(!mp3.is_empty());

        let url = to_url(CAMBRIDGE_URL, "bail out");
        let found = download_from_url(&url).await;
        assert!(found.is_ok());
        let html = found.unwrap();
        let mp3 = mp3_element(r#"source[type="audio/mpeg"]"#, &html).unwrap();
        println!("mp3 element: {}", mp3);
        assert!(!mp3.is_empty());
    }

    #[tokio::test]
    async fn definition_test() {
        let url = to_url(CAMBRIDGE_URL, "bail out");
        let html = download_from_url(&url).await.unwrap();
        let definition = first_element(r#"div[class="def ddef_d db"]"#, &html, true);
        assert!(definition.is_some());
        let inner_txt = definition.unwrap();
        println!("inner html: {}", inner_txt);
        println!("definition: {}", description(inner_txt).unwrap());
    }

    #[tokio::test]
    async fn cambridge_definitions_test() {
        let url = to_url(CAMBRIDGE_URL, "correct");
        let html = download_from_url(&url).await.unwrap();
        let definitions = elements(r#"div[class="def ddef_d db"]"#, &html, true);
        assert!(!definitions.is_empty());
        assert_eq!(9, definitions.len());
        for d in definitions {
            println!("-> {}", description(d).unwrap());
        }
    }

    #[test]
    fn test_not_found() {
        let lower = String::from(PAGE_NOT_FOUND).to_lowercase();
        let mut found = false;
        for i in 0..NOT_FOUND.len() {
            if lower.contains(NOT_FOUND[i]) {
                found = true;
                break;
            }
        }

        assert!(found);
    }

    #[tokio::test]
    async fn test_collins_url() {
        let cmd = format!("wget --user-agent=\"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15\" https://www.collinsdictionary.com/dictionary/english/{} -O download/tmp/collins/{}.html", "correct", "correct");
        println!("cmd: {}", cmd);
        let command = std::process::Command::new("pwd").output().unwrap();
        println!("output: {:?}", String::from_utf8(command.stdout).unwrap());

        std::process::Command::new("mkdir")
            .arg("-p")
            .arg("download/tmp/collins")
            .status()
            .expect("failed to execute mkdir");
        std::process::Command::new("mkdir")
            .arg("-p")
            .arg("download/tmp/cambridge")
            .status()
            .expect("failed to execute mkdir");
        std::process::Command::new("mkdir")
            .arg("-p")
            .arg("download/tmp/oxford")
            .status()
            .expect("failed to execute mkdir");

        std::process::Command::new("wget")
            .arg("--user-agent")
            .arg("'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15'")
            .arg("https://www.collinsdictionary.com/dictionary/english/rampage")
            .arg("-O")
            .arg("download/tmp/collins/rampage.html")
            .status()
            .expect("failed to execute wget");

        std::process::Command::new("wget")
            .arg("--user-agent")
            .arg("'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15'")
            .arg("https://dictionary.cambridge.org/dictionary/english/correct")
            .arg("-O")
            .arg("download/tmp/cambridge/correct.html")
            .status()
            .expect("failed to execute wget");

        std::process::Command::new("wget")
            .arg("--user-agent")
            .arg("'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15'")
            .arg("https://www.oxfordlearnersdictionaries.com/definition/english/correct")
            .arg("-O")
            .arg("download/tmp/oxford/correct.html")
            .status()
            .expect("failed to execute wget");
    }

    #[tokio::test]
    async fn oxford_redirect_test() {
        let word = "unrelenting";
        let url = to_url(OXFORD_URL, word);

        let html = download_from_url(&url).await.unwrap();

        let elements = elements(r#"link[rel="canonical"]"#, &html, false);
        assert!(!elements.is_empty());
        let txt = elements.get(0).unwrap();
        println!("found: {:?}", txt);
        let arr = txt
            .split_whitespace()
            .filter(|s| s.starts_with("href"))
            .at_most_one()
            .unwrap()
            .unwrap();
        println!("href: {:?}", arr);
        let url = arr
            .split("\"")
            .filter(|s| s.starts_with("http"))
            .at_most_one()
            .unwrap()
            .unwrap();
        println!("url: {:?}", url);
        assert!(url.ends_with(word))
    }

    #[tokio::test]
    async fn oxford_first_element_test() {
        let url = to_url(OXFORD_URL, "unrelenting");
        let html = download_from_url(&url).await.unwrap();
        let first = first_element(r#"link[rel="canonical"]"#, &html, false);
        assert!(first.is_some());
        let txt = first.unwrap();
        assert!(txt.contains(
            r#""https://www.oxfordlearnersdictionaries.com/definition/english/unrelenting""#
        ))
    }

    #[test]
    fn download_sth() {
        std::process::Command::new("wget")
        .arg("--user-agent")
        .arg("'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15'")
        .arg("https://www.oxfordlearnersdictionaries.com/definition/english/correct_2")
        .arg("-O")
        .arg("download/tmp/oxford/correct_2.html")
        .status()
        .expect("failed to execute wget");
    }

    #[test]
    fn to_next_word2_test() {
        let word = "correct";
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
                r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_2"#
            )),
            check_for_more(word, &html)
        );
    }

    #[tokio::test]
    async fn merge_correct2_test() {
        let res = merge_definitions(
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_1"#,
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_2"#,
            r#""#,
        )
        .await;
        assert!(res.is_some());
        println!("found: {:?}", res.unwrap());
    }

    #[tokio::test]
    async fn merge_correct3_test() {
        let res = merge_definitions(
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_1"#,
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_2"#,
            r#"https://www.oxfordlearnersdictionaries.com/definition/english/correct_3"#,
        )
        .await;
        assert!(res.is_some());
        println!("found: {:?}", res.unwrap());
    }
}
