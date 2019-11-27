use futures::future::join_all;

mod canonical_json;
mod client;
mod kinto_http;
mod signatures;

use client::Client;
use signatures::Verifier;

const SERVER_PROD: &'static str = "https://firefox.settings.services.mozilla.com/v1";

#[tokio::main]
async fn main() {
    let client = Client::new(SERVER_PROD.to_string());

    let entries = client.poll_changes().await.unwrap();

    let futures = entries.iter().map(|(bid, cid, timestamp)| {
        client.fetch_collection(bid.to_owned(), cid.to_owned(), timestamp.to_owned())
    });
    let results = join_all(futures).await;

    let verifier = Verifier::new();
    for result in results {
        let dataset = result.unwrap();
        let valid = verifier.verify(&dataset);
        println!("{}/{}: {}", dataset.bid, dataset.cid, valid);
    }
}
