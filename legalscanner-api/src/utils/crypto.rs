use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use base64::{engine::general_purpose, Engine};
use rand::Rng;

const API_KEY_LENGTH: usize = 32;
const API_KEY_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// Generate a random API key
pub fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let key: String = (0..API_KEY_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..API_KEY_CHARS.len());
            API_KEY_CHARS[idx] as char
        })
        .collect();
    format!("lgs_{}", key) // lgs = legal scanner
}

/// Hash an API key using Argon2
pub fn hash_api_key(key: &str, salt: &str) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();

    // Convert salt string to base64-compatible format
    // Take first 16 bytes of the salt string and encode to base64
    let salt_bytes: Vec<u8> = salt.bytes().take(16).chain(std::iter::repeat(0)).take(16).collect();
    let salt_b64 = general_purpose::STANDARD.encode(&salt_bytes);

    // SaltString expects 22 characters of base64
    let salt_b64_truncated = format!("{:.<22}", salt_b64.chars().take(22).collect::<String>());

    let salt_string = SaltString::from_b64(&salt_b64_truncated)?;
    let password_hash = argon2.hash_password(key.as_bytes(), &salt_string)?;
    Ok(password_hash.to_string())
}

/// Verify an API key against a hash
pub fn verify_api_key(key: &str, salt: &str) -> Result<String, argon2::password_hash::Error> {
    // For verification, we just hash and compare
    hash_api_key(key, salt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key();
        assert!(key.starts_with("lgs_"));
        assert_eq!(key.len(), 4 + API_KEY_LENGTH);
    }

    #[test]
    fn test_hash_and_verify() {
        let key = "test_api_key_123";
        let salt = "test_salt_for_hashing";

        let hash1 = hash_api_key(key, salt).unwrap();
        let hash2 = hash_api_key(key, salt).unwrap();

        // Same key and salt should produce same hash
        assert_eq!(hash1, hash2);
    }
}
