use serde::Deserialize;
use serde_json;

const SERVER_PROD: &'static str = "https://firefox.settings.services.mozilla.com/v1";

type Record = serde_json::Value;

#[derive(Deserialize)]
struct RecordsResponse {
    data: Vec<Record>,
}

struct RecordsListResult {
    records: Vec<Record>,
    timestamp: String,
}

async fn get_records(
    server: &str,
    bucket: &str,
    collection: &str,
) -> Result<RecordsListResult, reqwest::Error> {
    let url: String = format!(
        "{}/buckets/{}/collections/{}/records",
        server, bucket, collection
    );
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

    Ok(RecordsListResult {
        records: result.data,
        timestamp: timestamp,
    })
}

#[tokio::main]
async fn main() {
    let result = get_records(SERVER_PROD, "monitor", "changes")
        .await
        .unwrap();

    println!("Last modified {}", result.timestamp);

    for entry in result.records {
        let bucket = entry["bucket"].as_str().unwrap();
        let collection = entry["collection"].as_str().unwrap();
        if bucket.to_string().ends_with("preview") {
            continue;
        }
        let col_result = get_records(SERVER_PROD, bucket, collection).await.unwrap();
        println!(
            "{}/{}: {} records.",
            bucket,
            collection,
            col_result.records.len()
        );
    }
}
