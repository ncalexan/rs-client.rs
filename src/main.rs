use serde::Deserialize;
use serde_json;

const SERVER_PROD: &'static str = "https://firefox.settings.services.mozilla.com/v1";

#[derive(Debug, Deserialize)]
struct PluralResponse {
    data: serde_json::Value,
}

async fn get_records(
    server: String,
    bucket: String,
    collection: String,
) -> Result<PluralResponse, reqwest::Error> {
    let url: String = format!(
        "{}/buckets/{}/collections/{}/records",
        server, bucket, collection
    );
    println!("Fetch {:#?}...", url);
    let resp = reqwest::get(&url).await?;
    let body = resp.text().await?;
    let result: PluralResponse = serde_json::from_str(&body).unwrap();
    Ok(result)
}

#[tokio::main]
async fn main() {
    let records = get_records(SERVER_PROD.into(), "main".into(), "cfr".into())
        .await
        .unwrap();
    println!("{:#?}", records);
}
