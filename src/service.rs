use tokio::time::Duration;

#[derive(Debug)]
pub enum SignServiceError {

}

#[derive(Default)]
pub struct SignService {

}

impl SignService {
    pub async fn sign_message(&self, message: &str, key: &str) -> Result<String, SignServiceError> {
        
        tokio::time::sleep(Duration::from_millis(1000)).await;
        println!("sign_message: {}", message);
        //Event::message(r##"{"id": 58, "msg": "text"}"##.to_string());

        Ok(String::from("message signed"))
    }
}