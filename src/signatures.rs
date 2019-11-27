use crate::canonical_json::serialize;
use crate::client::Collection;

pub struct Verifier {}

impl Verifier {
    pub fn new() -> Self {
        Verifier {}
    }

    pub fn verify(&self, collection: &Collection) -> bool {
        let signature = collection.metadata["signature"]["signature"]
            .as_str()
            .unwrap()
            .to_string();
        let canonical: Vec<String> = collection.records.iter().map(|r| serialize(r)).collect();

        let serialized = format!(
            "{{\"data\":[{}],\"last_modified\":{}}}",
            canonical.join(","),
            collection.timestamp
        );

        signature == serialized
    }
}
