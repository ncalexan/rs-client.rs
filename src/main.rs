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
    println!(
        "Download {:?} bytes...",
        resp.headers().get("content-length").unwrap()
    );
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

    let entries: Vec<(String, String)> = records
        .iter()
        .filter_map(|entry| {
            let bid = entry["bucket"].as_str().unwrap().to_string();
            let cid = entry["collection"].as_str().unwrap().to_string();
            if !bid.ends_with("preview") {
                Some((bid, cid))
            } else {
                None
            }
        })
        .collect();

    let futures = entries
        .iter()
        .map(|(bid, cid)| get_records(SERVER_PROD.to_string(), bid.to_owned(), cid.to_owned()));

    let results = join_all(futures).await;

    for ((bucket, collection), ref result) in entries.iter().zip(results) {
        let (records, timestamp) = result.as_ref().unwrap();
        println!(
            "{}/{}: {} records ({})",
            bucket,
            collection,
            records.len(),
            timestamp
        );
    }
}
