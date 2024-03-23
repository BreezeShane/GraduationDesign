use std::env;
use sha2::Sha384;
use chrono::{TimeZone, Utc};
use deadpool_postgres::Pool;
use hmac::{digest::KeyInit, Hmac};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use jwt::{AlgorithmType, Error, Header, SignWithKey, Token, VerifyWithKey};

use data_encoding::HEXUPPER;
use ring::error::Unspecified;
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use std::num::NonZeroU32;

use axum::{
    Json,
    middleware::Next, 
    extract::Request, 
    response::Response, 
    extract::{Path, State}, 
    http::{HeaderMap, StatusCode}, 
};

const JWT_EXPIRATION: i64 = 3900;
const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    user_name: String,
    // permissions: i8,
    expire_on: usize,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "Account")]
pub struct Account {
    id: u32,
    nick_name: String,
    password_salt: String,
    password_hash: String,
    email: String,
    permissions: i8,
    contribution: i16,
    available: bool,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "RequestAccount")]
pub struct RequestAccount {
    id: u32,
    nick_name: String,
    password: String,
    email: String,
    permissions: i8,
    contribution: i16,
    available: bool,
}

// Possible to create struct Config to maintain envs.
fn key_from_secret() -> Result<Hmac<Sha384>, Error> {
    let secret = env::var("JWT_SECRET")
        .unwrap_or("Usagi Peropero!".to_string());
    let key = Hmac::new_from_slice(secret.as_bytes()).unwrap();
    Ok(key)
}

fn generate_jwt(claims: Claims) 
    -> Result<String, Error> {
    let key = key_from_secret().unwrap();
    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };
    let token = Token::new(header, claims).sign_with_key(&key)?;

    Ok(token.as_str().to_string())
}

pub fn verify_jwt(token: String) -> Result<Claims, Error> {
    let key = key_from_secret().unwrap();
    let verify: Result<Token<Header, Claims, _>, _>
        = token.verify_with_key(&key);
    match verify {
        Ok(token) => {
            let claims: Claims = token.claims().clone();
            let expiry = 
                Utc.timestamp_opt(claims.expire_on as i64, 0).unwrap();
            let now = Utc::now();
            if now > expiry {
                return Err(Error::InvalidSignature);
            }
            Ok(claims)
        }
        Err(err) => Err(err),
    }
}

pub async fn handler_sign_in(
    State(pool): State<Pool>,
    Path(sign_in_form): Path<RequestAccount>
) -> Result<axum::Json<String>, (StatusCode, String)> {
    let client = pool.get().await.unwrap();
    let user_request: RequestAccount = sign_in_form;

    let query_statement = client
    .prepare("
        SELECT nick_name, password_salt, password_hash, email, available FROM account WHERE email=$1 ORDER BY id DESC LIMIT 1;
    ")
    .await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let account: Account = client
    .query(&query_statement, &[&user_request.email])
    .await
    .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))? 
    .iter()
    .map(|row| Account::from_row_ref(row).unwrap())
    // .map(|row: &tokio_postgres::Row| Account {
    //     id: row.get("id"),
    //     nick_name: row.get("nick_name"),
    //     email: row.get("email"),
    //     contribution: row.get("contribution"),
    //     available: row.get("available"),
    // })
    .collect::<Vec<Account>>()
    .pop()
    .ok_or((StatusCode::NOT_FOUND, format!("Couldn't find account #{}", user_request.email)))?;

    if !account.available {
        return Err((StatusCode::FORBIDDEN, "The account has been forbidden!".to_string()));
    }

    if !password_authentificate(
            user_request.password, 
            account.password_salt, 
            account.password_hash
    ) {
        return Err((StatusCode::FORBIDDEN, "Wrong email or password!".to_string()));
    }
    
    let claims = Claims {
        sub: account.email,
        user_name: account.nick_name,
        // permissions: account.permissions,
        expire_on: (Utc::now().timestamp() + JWT_EXPIRATION) as usize
    };

    let token = generate_jwt(claims).unwrap();
    Ok(Json(token))
}

pub async fn handler_sign_up(
    State(pool): State<Pool>,
    Path(sign_up_form): Path<RequestAccount>
) -> Result<axum::Json<String>, (StatusCode, String)> {
    let client = pool.get().await.unwrap();

    let user_request = sign_up_form;

    let query_statement = client
    .prepare("
        SELECT nick_name, email, available FROM account WHERE email=$1 ORDER BY id DESC LIMIT 1;
    ")
    .await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let account = client
    .query(&query_statement, &[&user_request.email])
    .await
    .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))? 
    .iter()
    .map(|row| Account::from_row_ref(row).unwrap())
    .collect::<Vec<Account>>()
    .pop();

    match account {
        None => {
            let contribute: i16 = 0;
            let available = true;
            let permissions = 0b00001i8;
            let (passwd_salt, passwd_hash) = 
                encrypt(user_request.password);
            
            let insert_statement = client
            .prepare("
                INSERT INTO account (nick_name, password_salt, password_hash, email, contribution, available, permissions)
                VALUES
                ($1, $2, $3, $4, $5, $6, $7)
            ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

            let rows = client
            .execute(&insert_statement, 
                &[
                    &user_request.nick_name, &passwd_salt, &passwd_hash, &user_request.email, 
                    &contribute, &available, &permissions
                ])
            .await
            .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;
            
            if rows < 1 {
                return Err((StatusCode::NOT_MODIFIED, "Register account failed".to_string()));
            }
            Ok(Json("Succeeded to sign up!".to_string()))
        },
        Some(_) => {
            Err((StatusCode::CONFLICT, "The email has been used!".to_string()))
        }
    }
}

pub async fn handler_sign_out() {

}

pub async fn middleware_authorize(
    headers: HeaderMap,
    request: Request,
    next: Next
) -> Result<Response, (StatusCode, String)> {
    match get_token(&headers) {
        Some(token) if token_is_valid(token) => {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err((StatusCode::UNAUTHORIZED, "Token is expired or invalid!".to_string()))
    }
}

fn get_token(headers: &HeaderMap) -> Option<&str> {
    let request_header = headers.get("token").unwrap();
    Some(request_header.to_str().unwrap())
}

fn token_is_valid(token_str: &str) -> bool {
    match verify_jwt(token_str.to_string()) {
        Ok(_) => true,
        _ => false
    }
}

fn encrypt(password_string: String) -> (String, String) {
    let n_iter = NonZeroU32::new(100_000).unwrap();
    let rng = rand::SystemRandom::new();
    
    let mut salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut salt).unwrap();
    
    let password = password_string.as_str();
    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        n_iter,
        &salt,
        password.as_bytes(),
        &mut pbkdf2_hash,
    );

    (
        HEXUPPER.encode(&salt), 
        HEXUPPER.encode(&pbkdf2_hash)
    )
}

fn password_authentificate(password_string: String, salt_string: String, pbkdf2_hash_string: String) -> bool {
    let n_iter = NonZeroU32::new(100_000).unwrap();
    let salt = HEXUPPER.decode(salt_string.as_bytes()).unwrap();
    let pbkdf2_hash = HEXUPPER.decode(pbkdf2_hash_string.as_bytes()).unwrap();

    let password = password_string.as_str();

    let authentification_result: Result<(), Unspecified> = pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512,
        n_iter,
        &salt,
        password.as_bytes(),
        &pbkdf2_hash
    );

    match authentification_result {
        Ok(_) => true,
        Err(_) => false
    }
}