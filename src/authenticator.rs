use std::env;
use std::ops::BitAnd;
use axum::extract::Path;
use axum::http::HeaderValue;
use sha2::Sha384;
use chrono::{TimeZone, Local};
use deadpool_postgres::Pool;
use hmac::{digest::KeyInit, Hmac};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use jwt::{AlgorithmType, Error, Header, SignWithKey, Token, VerifyWithKey};
use crate::config::{JWT_EXPIRATION, JWT_REFRESH_PERIOD};
use crate::MultiState;

use data_encoding::HEXUPPER;
use ring::error::Unspecified;
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use std::num::NonZeroU32;

use axum::{
    Form,
    Json,
    middleware::Next,
    extract::Request,
    response::Response,
    extract::State,
    http::{HeaderMap, StatusCode},
};

const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;

#[macro_export]
macro_rules! back_to_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<i16> for $name {
            type Error = ();

            fn try_from(v: i16) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as i16 => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}

back_to_enum! {
    /// Manage_Feedback Manage_User Manage_Model Train&Upload_Model&View_Training_Effect Common
    /// 0/1             0/1         0/1          0/1                                     0/1
    /// Permission: Enum
    /// Permission::<Role>: isize
    /// Each bit means status of the matched permission.
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Role {
        UserAdmin   =   0b11001isize,
        ModelAdmin  =   0b00111isize,
        CommonUser  =   0b00001isize,
        SuperRoot   =   0b11111isize,
    }
}

pub fn role_to_string(permissions: i16) -> String {
    let role: Role = permissions.try_into().unwrap();
    return match role {
        Role::UserAdmin => "User Administrator".to_string(),
        Role::CommonUser => "Common User".to_string(),
        Role::ModelAdmin => "Model Administrator".to_string(),
        Role::SuperRoot => "Super Root".to_string(),
    }
}

pub enum Permission {
    MngFeedBack =   0b10000isize,
    MngUsr      =   0b01000isize,
    MngModel    =   0b00100isize,
    TUV         =   0b00010isize,
    Common      =   0b00001isize
}

impl BitAnd<Permission> for Role {
    type Output = bool;
    fn bitand(self, rhs: Permission) -> Self::Output {
        let role = self as isize;
        let permission = rhs as isize;
        role & permission == permission
    }
}

// impl TryFrom<i8> for Role {
//     type Error = ();

//     fn try_from(value: i8) -> Result<Self, Self::Error> {
//         match value {
//             x if x == Role::CommonUser as i8 => Ok(Role::CommonUser),
//             x if x == Role::UserAdmin as i8 => Ok(Role::UserAdmin),
//             x if x == Role::ModelAdmin as i8 => Ok(Role::ModelAdmin),
//             _ => Err(())
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    user_email: String,
    user_name: String,
    expire_on: usize,
}

#[derive(Serialize, Deserialize, PostgresMapper, Clone, Debug)]
#[pg_mapper(table = "Account")]
struct AuthenAccount {
    nick_name: String,
    password_salt: String,
    password_hash: String,
    email: String,
    available: bool,
}

#[derive(Serialize, Deserialize, PostgresMapper, Clone, Debug)]
#[pg_mapper(table = "Account")]
pub struct ProofAccount {
    pub email: String,
    pub permissions: i16,
    pub available: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RequestAccountForSignIn {
    useremail: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestAccountForSignUp {
    username: String,
    password: String,
    repassword: String,
    email: String,
}

pub async fn check_permission (connection: &Pool, useremail: &String, needed_permission: Permission) -> Result<bool, (StatusCode, String)> {
    let client = connection.get().await.unwrap();

    let auth_statement = client
    .prepare("
        SELECT email, permissions, available FROM account WHERE email=$1;
    ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    let current_user = client
    .query(&auth_statement, &[&useremail])
    .await
    .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    .iter()
    .map(|row| ProofAccount::from_row_ref(row).unwrap())
    .collect::<Vec<ProofAccount>>()
    .pop()
    .ok_or((StatusCode::NOT_FOUND, format!("Couldn't find account: {:?}", useremail)))?;

    // if !check_permission(&current_user, Permission::Common) {
    //     return  Err(
    //         (StatusCode::FORBIDDEN, "Not permitted!".to_string())
    //     );
    // }
    let role: Role = current_user.permissions.try_into().unwrap();
    Ok(role & needed_permission)
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

pub fn verify_jwt(token: &String) -> Result<Claims, Error> {
    let key = key_from_secret().unwrap();
    let verify: Result<Token<Header, Claims, _>, _>
        = token.verify_with_key(&key);
    match verify {
        Ok(token) => {
            let claims: Claims = token.claims().clone();
            let expiry =
                Local.timestamp_opt(claims.expire_on as i64, 0).unwrap();
            let now = Local::now();
            if now > expiry {
                return Err(Error::InvalidSignature);
            }
            Ok(claims)
        }
        Err(err) => Err(err),
    }
}

pub async fn handler_sign_in<'a>(
    State(multi_state): State<MultiState>,
    Form(sign_in_form): Form<RequestAccountForSignIn>
) -> Result<(HeaderMap, &'a str), (StatusCode, String)> {
    let client = multi_state.db_pool.get().await.unwrap();
    let user_request: RequestAccountForSignIn = sign_in_form;

    let query_statement = client
    .prepare("
        SELECT nick_name, password_salt, password_hash, email, available FROM account WHERE email=$1;
    ")
    .await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let account: AuthenAccount = client
    .query(&query_statement, &[&user_request.useremail])
    .await
    .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    .iter()
    .map(|row| AuthenAccount::from_row_ref(row).unwrap())
    // .map(|row: &tokio_postgres::Row| Account {
    //     id: row.get("id"),
    //     nick_name: row.get("nick_name"),
    //     email: row.get("email"),
    //     contribution: row.get("contribution"),
    //     available: row.get("available"),
    // })
    .collect::<Vec<AuthenAccount>>()
    .pop()
    .ok_or((StatusCode::NOT_FOUND, format!("Couldn't find account #{}", user_request.useremail)))?;

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
        user_email: account.email,
        user_name: account.nick_name,
        // permissions: account.permissions,
        expire_on: (Local::now().timestamp() + JWT_EXPIRATION) as usize
    };

    let token = generate_jwt(claims).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("auth-token",
        HeaderValue::from_str(token.as_str()).unwrap());
    Ok((headers, "Succeeded to sign in!"))
}

pub async fn handler_sign_up(
    State(multi_state): State<MultiState>,
    Form(sign_up_form): Form<RequestAccountForSignUp>
) -> Result<axum::Json<String>, (StatusCode, String)> {
    let client = multi_state.db_pool.get().await.unwrap();

    let user_request = sign_up_form;
    if user_request.password != user_request.repassword {
        return Err((StatusCode::NON_AUTHORITATIVE_INFORMATION, "The passwords should be the same!".to_string()))
    }

    let query_statement = client
    .prepare("
        SELECT email, permissions, available FROM account WHERE email=$1;
    ")
    .await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let account = client
    .query(&query_statement, &[&user_request.email])
    .await
    .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    .iter()
    .map(|row| ProofAccount::from_row_ref(row).unwrap())
    .collect::<Vec<ProofAccount>>()
    .pop();

    match account {
        None => {
            let contribute: i16 = 0;
            let available = true;
            let permissions = Role::CommonUser as i16;
            let (passwd_salt, passwd_hash) =
                encrypt(user_request.password);

            let insert_statement = client
            .prepare("
                INSERT INTO account (nick_name, password_salt, password_hash, email, contribution, available, permissions)
                VALUES
                ($1, $2, $3, $4, $5, $6, $7)
            ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

            // let rows = client
            // .execute(&insert_statement,
            //     &[ &user_request.username, &passwd_salt, &passwd_hash, &user_request.email,
            //         &contribute, &available, &permissions]
            // )
            // .await
            // .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;
            let rows = client
            .execute(&insert_statement,
                &[ &user_request.username, &passwd_salt, &passwd_hash, &user_request.email,
                    &contribute, &available, &permissions]
            )
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

// pub async fn handler_sign_out() -> Result<axum::Json<String>, (StatusCode, String)> {
//     Err((StatusCode::NOT_IMPLEMENTED, "Not implemented API!".to_string()))
// }

pub async fn middleware_authorize(
    headers: HeaderMap,
    request: Request,
    next: Next
) -> Result<Response, (StatusCode, String)> {
    let token_opt = get_token(&headers);
    if let None = token_opt {
        return Err((StatusCode::UNAUTHORIZED, "Token is invalid!".to_string()))
    }
    let token = token_opt.unwrap();
    let parse_result = verify_jwt(&token);
    if let Err(err) = parse_result {
        return Err(
            (StatusCode::UNAUTHORIZED,
                format!("Token is invalid or expired! Error: {err}")));
    }

    let mut response = next.run(request).await;

    let mut claims = parse_result.unwrap();
    if claims.expire_on as i64 - Local::now().timestamp() <= JWT_REFRESH_PERIOD {
        claims.expire_on = (Local::now().timestamp() + JWT_EXPIRATION) as usize;
        let new_token = generate_jwt(claims).unwrap();
        response.headers_mut()
        .insert("auth-token",
            HeaderValue::from_str(new_token.as_str()).unwrap());
    }

    Ok(response)
}

fn get_token(headers: &HeaderMap) -> Option<String> {
    let __token_header_value = headers.get("auth-token");
    if let None = __token_header_value {
        return None;
    }
    let __token_str = __token_header_value.unwrap().to_str().unwrap();
    let __token = __token_str.to_string();
    Some(__token)
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

pub async fn handler_transfer_permission_to_role(
    State(multi_state): State<MultiState>,
    Path(useremail): Path<String>
) -> Result<String, (StatusCode, String)> {
    let client = multi_state.db_pool.get().await.unwrap();

    let query_statement = client
    .prepare("
        SELECT email, permissions, available FROM account WHERE email=$1;
    ")
    .await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let account = client
    .query(&query_statement, &[&useremail])
    .await
    .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    .iter()
    .map(|row| ProofAccount::from_row_ref(row).unwrap())
    .collect::<Vec<ProofAccount>>()
    .pop().unwrap();

    let role = role_to_string(account.permissions);
    return Ok(role);
}