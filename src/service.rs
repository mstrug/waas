use tokio::time::Duration;
use k256::{ecdsa::{SigningKey, Signature, signature::Signer, VerifyingKey, signature::Verifier}};
use rand_core::OsRng; 
use base64::prelude::*;


#[derive(Debug)]
pub enum SignServiceError {
    KeyError
}

#[derive(Default)]
pub struct SignService {

}

impl SignService {

    pub fn generate_key(&self) -> Vec<u8> {
        let signing_key = SigningKey::random(&mut OsRng); 
        signing_key.to_bytes().to_vec()
    }

    pub async fn sign_message(&self, message: &str, key: &[u8]) -> Result<String, SignServiceError> {
        let msg = message.as_bytes();

        let signing_key = SigningKey::from_bytes(key).map_err(|_| SignServiceError::KeyError)?;
        let signature: Signature = signing_key.sign(msg);

        tokio::time::sleep(Duration::from_millis(1000)).await;

        Ok(BASE64_STANDARD.encode(signature))
    }
}