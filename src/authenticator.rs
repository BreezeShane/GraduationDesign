use std::env;
// use axum::Error;
// use hmac::{Hmac, digest::{InvalidLength, KeyInit}};
use serde::{Deserialize, Serialize};
use hmac::{Hmac, digest::KeyInit};
// use jwt::{AlgorithmType, Error, Header, SignWithKey, Token, Unsigned};
use jwt::{AlgorithmType, Error, Header, SignWithKey, Token};
use sha2::Sha384;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    user_name: String,
    expire_on: usize,
}

// Possible to create struct Config to maintain envs.
pub fn key_from_secret() -> Result<Hmac<Sha384>, Error> {
    let secret = env::var("JWT_SECRET")
        .unwrap_or("Usagi Peropero!".to_string());
    let key = Hmac::new_from_slice(secret.as_bytes()).unwrap();
    Ok(key)
}

pub fn generate_jwt(key: Hmac<Sha384>, claims: Claims) 
    -> Result<String, Error> {
    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };
    let token = Token::new(header, claims).sign_with_key(&key)?;

    Ok(token.as_str().to_string())
}

pub async fn authorize() {

}

pub async fn authenticate() {

}

pub async fn handler_sign_in() {
    
}

pub async fn handler_sign_up() {

}

pub async fn handler_sign_out() {

}
