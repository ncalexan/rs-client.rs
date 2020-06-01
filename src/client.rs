use crate::kinto_http::{get_changeset, get_records, KintoError, KintoObject};

#[derive(Debug)]
pub enum ClientError {}

impl From<KintoError> for ClientError {
    fn from(err: KintoError) -> Self {
        err.into()
    }
}

pub struct Collection {
    pub bid: String,
    pub cid: String,
    pub metadata: KintoObject,
    pub records: Vec<KintoObject>,
    pub timestamp: i64,
}

pub struct Client {
    server: String,
}

impl Client {
    pub fn new(server: String) -> Self {
        Client {
            server: server.to_owned(),
        }
    }

    pub fn poll_changes(&self) -> Result<Vec<(String, String, u64)>, ClientError> {
        let (records, _) = get_records(
            self.server.to_owned(),
            "monitor".to_string(),
            "changes".to_string(),
            0,
        )?;

        let entries: Vec<(String, String, u64)> = records
            .iter()
            .filter_map(|entry| {
                let bid = entry["bucket"].as_str().unwrap().to_string();
                let cid = entry["collection"].as_str().unwrap().to_string();
                let timestamp = entry["last_modified"].as_u64().unwrap();
                if !bid.ends_with("preview") {
                    Some((bid, cid, timestamp))
                } else {
                    None
                }
            })
            .collect();

        Ok(entries)
    }

    pub fn fetch_collection(
        &self,
        bid: String,
        cid: String,
        expected: u64,
    ) -> Result<Collection, ClientError> {
        let changeset = get_changeset(
            self.server.to_owned(),
            bid.to_owned(),
            cid.to_owned(),
            expected,
        )?;

        Ok(Collection {
            bid: bid.to_owned(),
            cid: cid.to_owned(),
            metadata: changeset.metadata.to_owned(),
            records: changeset.changes.to_owned(),
            timestamp: changeset.timestamp,
        })
    }
}
