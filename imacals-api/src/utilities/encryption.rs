use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, KeyInit};
use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};

// Derives a 32-byte AES-256 key by hashing the secret with SHA-256.
fn derive_key(secret: &str) -> [u8; 32] {
    let hash = Sha256::digest(secret.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    key
}

// Encrypts plaintext with AES-256-GCM. Returns base64(nonce || ciphertext).
pub fn encrypt(plaintext: &str, secret: &str) -> Result<String, String> {
    let key = derive_key(secret);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("invalid key: {e}"))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("encryption failed: {e}"))?;
    let mut blob = nonce.to_vec();
    blob.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&blob))
}

// Decrypts a blob produced by `encrypt`. Returns the original plaintext.
pub fn decrypt(encoded: &str, secret: &str) -> Result<String, String> {
    let blob = STANDARD.decode(encoded).map_err(|e| format!("base64 decode failed: {e}"))?;
    if blob.len() < 12 {
        return Err("ciphertext too short".into());
    }
    let (nonce_bytes, ciphertext) = blob.split_at(12);
    let nonce_arr: [u8; 12] = nonce_bytes.try_into().map_err(|_| "nonce slice wrong length".to_string())?;
    let key = derive_key(secret);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("invalid key: {e}"))?;
    let plaintext = cipher
        .decrypt(nonce_arr.as_ref().into(), ciphertext)
        .map_err(|e| format!("decryption failed: {e}"))?;
    String::from_utf8(plaintext).map_err(|e| format!("utf8 error: {e}"))
}
