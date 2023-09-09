use std::{collections::BTreeMap, fs};

use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;

pub struct TokenData {
    pub user_id: i64,
}

fn get_key_from_env() -> Option<Hmac<Sha256>> {
    let secret_file = std::env::var("JWT_SECRET_FILE").ok()?;
    let data = fs::read(secret_file).ok()?;
    Some(Hmac::new_from_slice(&data).ok()?)
}

pub fn sign(data: TokenData) -> String {
    let mut claims: BTreeMap<String, String> = BTreeMap::new();
    claims.insert("user_id".to_string(), data.user_id.to_string());

    let token = claims.sign_with_key(&get_key_from_env().unwrap()).unwrap();

    token
}

pub fn verify(jwt: &str) -> Option<TokenData> {
    let claims: BTreeMap<String, String> = jwt.verify_with_key(&get_key_from_env()?).ok()?;

    let user_id = claims.get("user_id")?.parse::<i64>().ok()?;

    Some(TokenData { user_id })
}
