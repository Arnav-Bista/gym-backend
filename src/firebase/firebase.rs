use std::fs;

use jsonwebtoken::{EncodingKey, encode, Header, Algorithm};
use reqwest::{self, Client};
use serde_json::{from_str, Value, json};

use crate::core_functions::error_logger::error_logger;

use super::service_key::ServiceKey;

pub struct Firebase {
    client: Client,
    db_url: String,
    service_key: ServiceKey,
    auth_token: Option<String>,
    jwt: String,
    exp: i64
}

impl Firebase {
    pub fn new(path_to_service_key: &str, db_url: String) -> Self {
        let key_contents = fs::read_to_string(path_to_service_key).unwrap();
        let key: Value = match from_str(&key_contents) {
            Ok(key) => key,
            Err(_) => {
                println!("Error parsing key");
                std::process::exit(1);
            }
        };

        let service_key = match ServiceKey::new(&key) {
            Some(key) => key,
            None => {
                println!("Key is missing fields.");
                std::process::exit(1);
            }
        };

        let (jwt, exp) = match Self::construct_jwt(&service_key) {
            Some(key) => key,
            None => {
                println!("Could not construct JWT. Check Key");
                std::process::exit(1);
            }
        };

        Self {
            client: Client::new(),
            db_url,
            service_key,
            jwt,
            auth_token: None,
            exp
        }
    }
    // Token + exp
    fn construct_jwt(key: &ServiceKey) -> Option<(String, i64)> {
        // Google OAuth2 Rest API
        let encoding_key = EncodingKey::from_rsa_pem(key.private_key().as_bytes()).ok()?;
        let iat = chrono::Utc::now().timestamp();
        let exp = iat + 3600; // 1 Hour Exp
        let token = encode(
            &Header::new(Algorithm::RS256),
            &json!({
                "iss": key.email(),
                "sub": key.email(),
                "scope" : "https://www.googleapis.com/auth/firebase.database https://www.googleapis.com/auth/userinfo.email",
                "aud": key.token_url(),
                "iat": iat,
                "exp": exp,
            }),
            &encoding_key
        ).ok()?;
        Some((token, exp))
    }

    fn set_jwt(&mut self) {
        let (jwt, exp) = match Self::construct_jwt(&self.service_key) {
            Some(key) => key,
            None => {
                println!("Could not construct JWT. Check Key");
                std::process::exit(1);
            }
        };
        self.jwt = jwt;
        self.exp = exp;
    }

    async fn set_auth_token(&mut self) -> Result<(),()> {
        let response = match self.client
            .post(self.service_key.token_url())
            .form(&[("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"), ("assertion", &self.jwt)])
            .send()
            .await {
                Ok(res) => res,
                Err(_) => {
                    error_logger("Error generating token").await;
                    return Err(());
                }
            };

        let json_res: Value = from_str(
            response
                .text()
                .await
                .expect("Unexpected Error")
                .as_str()
        ).expect("Unexpected response json parse error");

        // self.auth_token = Some(json_res["access_token"].as_str().expect("Unexpected error - unwrap on access token")
        //     .to_string());
        self.auth_token = match json_res["access_token"].as_str() {
            Some(token) => Some(token.to_string()),
            None => {
                error_logger("Access Token as_str error").await;
                dbg!(json_res);
                None
            }
        };
        if self.auth_token.is_none() {
            return Err(());
        }
        Ok(())
    }

    pub async fn handle_auth_token(&mut self) -> Result<(),()> {
        let now = chrono::Utc::now().timestamp();
        if self.exp < now || self.auth_token.is_none() {
            // Generate new jwt
            self.set_jwt();
            // get new key
            if self.set_auth_token().await.is_err() {
                return Err(());
            }
        }
        Ok(())
    }

    pub async fn update(&self, location: String, data: &str) {
        let auth_token = &self.auth_token.as_ref().unwrap();

        let response = match self.client
            .patch(format!("{}{}.json?access_token={}",self.db_url,location,auth_token))
            .body(data.to_string())
            .send()
            .await {
                Ok(data) => data,
                Err(_) => {
                    error_logger("Put error").await;
                    return;
                }
            };
        // dbg!(response.text().await.unwrap());
        println!("{} {}", response.status(), location);
    }

    pub async fn set(&self, location: String, data: &str) {
        let auth_token = &self.auth_token.as_ref().unwrap();

        let response = match self.client
            .put(format!("{}{}.json?access_token={}",self.db_url,location,auth_token))
            .body(data.to_string())
            .send()
            .await {
                Ok(data) => data,
                Err(_) => {
                    error_logger("Put error").await;
                    return;
                }
            };
        // dbg!(response.text().await.unwrap());
        println!("{} {}", response.status(), location);
    }

    pub async fn get(&self, location: String) -> Option<String> {
        let auth_token = &self.auth_token.as_ref().unwrap();
        let response = match self.client
        .get(format!("{}.json?access_token={}", location, auth_token))
            .send()
            .await {
                Ok(data) => data,
                Err(_) => {
                    error_logger("Get Error").await;
                    return None;
                }
            };
        Some(response.text().await.unwrap().to_string())
    }
}
