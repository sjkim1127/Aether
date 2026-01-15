use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use std::env;

/// Security utility for Aether Shield.
pub struct Shield;

impl Shield {
    /// Get the default key for the current machine.
    pub fn default_key() -> String {
        Self::get_machine_id()
    }

    /// Encrypt a prompt using a key derived from environment or provided key.
    pub fn encrypt(prompt: &str, key_str: &str) -> String {
        let key = Self::derive_key(key_str);
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Nonce::from_slice(b"aether_nonce"); // 12 bytes

        let ciphertext = cipher
            .encrypt(nonce, prompt.as_bytes())
            .expect("Encryption failed");

        general_purpose::STANDARD.encode(ciphertext)
    }

    /// Decrypt an encrypted prompt.
    pub fn decrypt(encrypted_prompt: &str, key_str: &str) -> Result<String, String> {
        let key = Self::derive_key(key_str);
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Nonce::from_slice(b"aether_nonce");

        let ciphertext = general_purpose::STANDARD
            .decode(encrypted_prompt)
            .map_err(|e| e.to_string())?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_slice())
            .map_err(|e| e.to_string())?;

        String::from_utf8(plaintext).map_err(|e| e.to_string())
    }

    /// Helper to derive a 32-byte key from a string.
    fn derive_key(key_str: &str) -> [u8; 32] {
        let mut key = [0u8; 32];
        let bytes = key_str.as_bytes();
        for i in 0..32 {
            if i < bytes.len() {
                key[i] = bytes[i];
            } else {
                key[i] = (i as u8).wrapping_mul(0xAF); // Padding
            }
        }
        key
    }

    /// Get current machine ID for dynamic key generation.
    /// (Simplified implementation for portability)
    pub fn get_machine_id() -> String {
        // Envs that might identify the user/machine
        let username = env::var("USERNAME").or_else(|_| env::var("USER")).unwrap_or_else(|_| "unknown".to_string());
        let computername = env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string());
        format!("{}-{}", username, computername)
    }
}
