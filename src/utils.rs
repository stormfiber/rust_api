use std::time::Duration;

pub mod image;

pub async fn keep_alive() {
    let url = "https://discardapi.dpdns.org";
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(Duration::from_secs(300));
    loop {
        interval.tick().await;
        let _ = client.get(url).send().await;
    }
}
