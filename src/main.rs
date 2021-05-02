use std::env;
use reqwest::{Client, Url};
use select::document::Document;
use select::predicate::Name;
use url::ParseError;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let url = get_url()?;
    let url = Url::parse(&url).map_err(|err| err.to_string())?;
    let body = make_request(url.clone()).await?;

    for url in get_urls(&body, &url)? {
        println!("{}", url);
    }

    Ok(())
}

fn get_url() -> Result<String, String> {
    env::args().nth(1)
        .ok_or(String::from("Url should be passed as argument"))
}

async fn make_request(url: Url) -> Result<String, String> {
    let client = Client::new();

    let response = client.get(url)
        .send().await
        .map_err(|err| err.to_string())?;

    let body = response.text()
        .await
        .map_err(|err| err.to_string())?;

    Ok(body)
}

fn get_urls(body: &str, source_url: &Url) -> Result<Vec<Url>, String> {
    let document = Document::from_read(body.as_bytes())
        .map_err(|err| err.to_string())?;

    let urls = document.find(Name("a"))
        .filter_map(|node| node.attr("href"))
        .filter_map(|url| normalize_url(source_url, url))
        .collect();

    Ok(urls)
}

fn normalize_url(source_url: &Url, url: &str) -> Option<Url> {
    match Url::parse(url) {
        Ok(url) => Some(url),
        Err(ParseError::RelativeUrlWithoutBase) => {
            source_url.join(url).ok()
        }
        _ => None
    }
}


