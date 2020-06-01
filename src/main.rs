mod canonical_json;
mod client;
mod kinto_http;
mod signatures;

use client::{Client, Collection};
use signatures::Verifier;
use std::collections::HashMap;
pub use viaduct::{note_backend, set_backend};
pub use viaduct_reqwest::ReqwestBackend;

const SERVER_PROD: &'static str = "https://firefox.settings.services.mozilla.com/v1";

fn main() {
    set_backend(&ReqwestBackend).unwrap(); // XXX

    let client = Client::new(SERVER_PROD.to_string());

    let entries = client.poll_changes().unwrap();

    let results: Result<Vec<_>, _> = entries.into_iter().map(|(bid, cid, timestamp)| {
        client.fetch_collection(bid, cid, timestamp)
    }).collect();

    let datasets: Vec<Collection> = results.unwrap(); // XXX

    let verifier = Verifier::new();
    let verif_results = datasets.iter().map(|ref dataset| verifier.verify(&dataset));

    let failing: HashMap<String, String> = datasets
        .iter()
        .zip(verif_results)
        .filter_map(|(dataset, verif)| {
            Some((
                format!("{}/{}", dataset.bid, dataset.cid),
                match verif {
                    Ok(()) => "OK".to_string(),
                    Err(e) => format!("{:?}", e),
                },
            ))
        })
        .collect();

    println!("{:#?}", failing);
}
