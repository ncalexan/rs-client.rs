use crate::canonical_json::serialize;
use crate::client::Collection;

use openssl::x509::X509;
use signatory::{
    ecdsa::{curve::NistP384, FixedSignature},
    verify_sha384, EcdsaPublicKey, Signature as SignatorySignature,
};
use signatory_ring::ecdsa::P384Verifier;

use base64;
use reqwest;
use reqwest::Error as ReqwestError;
use serde_json::json;

type EcdsaPublicKeyP384 = EcdsaPublicKey<NistP384>;
type EcdsaSignatureP384 = FixedSignature<NistP384>;

pub enum SignatureError {}

impl From<ReqwestError> for SignatureError {
    fn from(err: ReqwestError) -> Self {
        err.into()
    }
}

impl From<signatory::error::Error> for SignatureError {
    fn from(err: signatory::error::Error) -> Self {
        err.into()
    }
}

impl From<openssl::error::ErrorStack> for SignatureError {
    fn from(err: openssl::error::ErrorStack) -> Self {
        err.into()
    }
}

pub struct Verifier {}

impl Verifier {
    pub fn new() -> Self {
        Verifier {}
    }

    pub async fn verify(&self, collection: &Collection) -> Result<(), SignatureError> {
        // Serialized data.
        let serialized = serialize(&json!({
            "data": collection.records,
            "last_modified": collection.timestamp
        }));
        let data = format!("Content-Signature:\x00{}", serialized);

        // Content signature.
        let b64_signature = collection.metadata["signature"]["signature"]
            .as_str()
            .unwrap()
            .to_string();
        let bytes_url = base64::decode_config(&b64_signature, base64::URL_SAFE).unwrap();
        let signature = EcdsaSignatureP384::from_bytes(&bytes_url).unwrap();

        // Certificate public key.
        let x5u = collection.metadata["signature"]["x5u"].as_str().unwrap();
        let resp = reqwest::get(&x5u.to_string()).await?;
        let pem = resp.bytes().await?;
        let cert = X509::from_pem(&pem)?;
        let public_key = &cert.public_key().unwrap();

        let pk = EcdsaPublicKeyP384::from_bytes(&public_key.to_bytes()).unwrap();

        // // Verify!
        let r = verify_sha384(&P384Verifier::from(&pk), &data.as_bytes(), &signature);
        match r {
            Ok(_) => Ok(()),
            Err(e) => Err(SignatureError::from(e)),
        }
    }
}
