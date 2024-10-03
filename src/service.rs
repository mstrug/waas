
#[derive(Debug)]
pub enum SignServiceError {

}

#[derive(Default)]
pub struct SignService {

}

impl SignService {
    pub async fn sign_message(&self, message: &str, key: &str) -> Result<String, SignServiceError> {
        Ok(String::from("message signed"))
    }
}