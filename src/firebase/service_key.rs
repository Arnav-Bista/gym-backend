use serde_json::Value;


pub struct ServiceKey {
    email: String,
    private_key: String,
    token_url: String,
}


impl ServiceKey {
    pub fn new(key: &Value) -> Option<Self> {
        Some(
            Self {
                email: key["client_email"].as_str()?.to_string(),
                private_key: key["private_key"].as_str()?.to_string(),
                token_url: key["token_uri"].as_str()?.to_string()
            }
        )
    }

    pub fn email(&self) -> &String {
        &self.email
    }
    pub fn private_key(&self) -> &String {
        &self.private_key
    }

    pub fn token_url(&self) -> &String {
        &self.token_url
    }
}
