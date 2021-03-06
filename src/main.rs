use std::env;
use reqwest::{Client, Url};
use select::document::Document;
use select::predicate::Name;
use url::ParseError;
use tokio::sync::mpsc;
use futures::stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

use std::collections::HashSet;
use std::time::Duration;
use std::cell::RefCell;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let url = get_url()?;
    let url = Url::parse(&url).map_err(|err| err.to_string())?;

    let (sender, receiver) = mpsc::channel::<Url>(1_000_000);
    let url_stream = ReceiverStream::new(receiver);

    sender.send(url).await.map_err(|err| err.to_string())?;

    let visited = &RefCell::new(HashSet::new());

    let sender = &sender;
    url_stream.for_each_concurrent(None,|url| async move {
        if let Ok(body) = make_request(url.clone()).await {
            println!("{}", url);
            if let Ok(urls) = get_urls(&body, &url) {
                for url in urls {
                    if !visited.borrow().contains(&url) {
                        visited.borrow_mut().insert(url.clone());
                        sender.send(url).await;
                    }
                }
            }
        }
    }).await;

    Ok(())
}

fn get_url() -> Result<String, String> {
    env::args().nth(1)
        .ok_or(String::from("Url should be passed as argument"))
}

async fn make_request(url: Url) -> Result<String, String> {
    let client = Client::new();

    let response = client.get(url)
        .timeout(Duration::from_secs(30))
        .send().await
        .map_err(|err| err.to_string())?;

    let body = response.text()
        .await
        .map_err(|err| err.to_string())?;

    Ok(body)
}

fn get_urls(body: &str, source_url: &Url) -> Result<HashSet<Url>, String> {
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


