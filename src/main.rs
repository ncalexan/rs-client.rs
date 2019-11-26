use futures::future::join_all;
use serde::Deserialize;
use serde_json;

const SERVER_PROD: &'static str = "https://firefox.settings.services.mozilla.com/v1";

type Record = serde_json::Value;

#[derive(Deserialize)]
struct RecordsResponse {
    data: Vec<Record>,
}

async fn get_records(
    server: String,
    bid: String,
    cid: String,
) -> Result<(Vec<Record>, String), reqwest::Error> {
    let url = format!("{}/buckets/{}/collections/{}/records", server, bid, cid);
    println!("Fetch {}...", url);
    let resp = reqwest::get(&url).await?;
    let timestamp = resp
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.text().await?;

    // Parse JSON response.
    let result: RecordsResponse = serde_json::from_str(&body).unwrap();

    Ok((result.data, timestamp))
}

#[tokio::main]
async fn main() {
    let (records, timestamp) = get_records(
        SERVER_PROD.to_string(),
        "monitor".to_string(),
        "changes".to_string(),
    )
    .await
    .unwrap();

    println!("Last modified {}", timestamp);

    let entries = records
        .into_iter()
        .map(|entry| {
            let bid = entry["bucket"].as_str().unwrap().to_string();
            let cid = entry["collection"].as_str().unwrap().to_string();
            (bid, cid)
        })
        .filter(|(bid, _)| !bid.ends_with("preview"));

    let futures = entries.map(|(bid, cid)| get_records(SERVER_PROD.to_string(), bid, cid));

    let results = join_all(futures).await;

    for result in results {
        let (records, timestamp) = result.unwrap();
        println!("{}, {}", records.len(), timestamp);
    }
}
