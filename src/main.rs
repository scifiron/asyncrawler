use std::env;
use reqwest::Client;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let url = get_url()?;
    let body = make_request(&url).await?;
    println!("{}", body);
    Ok(())
}

fn get_url() -> Result<String, String> {
    env::args().nth(1)
        .ok_or(String::from("Url should be passed as argument"))
}

async fn make_request(url: &str) -> Result<String, String> {
    let client = Client::new();

    let response = client.get(url)
        .send().await
        .map_err(|err| err.to_string())?;

    let body = response.text()
        .await
        .map_err(|err| err.to_string())?;

    Ok(body)
}
