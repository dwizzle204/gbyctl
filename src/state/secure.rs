//! Encryption helpers for persisted local state.

use aes_gcm_siv::aead::{Aead, KeyInit};
use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "gbyctl";
const STATE_KEY_ID: &str = "state-encryption-key";
// Bind ciphertext to this application/version context to prevent accidental
// cross-use with unrelated encrypted payloads.
const AAD: &[u8] = b"gbyctl-state-v1";

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedBlob {
    v: u8,
    nonce: String,
    ciphertext: String,
}

/// Encrypt plaintext bytes for local persistence.
pub fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>> {
    let key = load_or_create_key()?;
    let cipher = aes_gcm_siv::Aes256GcmSiv::new_from_slice(&key)
        .map_err(|_| anyhow::anyhow!("invalid AES key length"))?;

    // Fresh nonce per record is required for AEAD safety.
    let mut nonce_bytes = [0_u8; 12];
    getrandom::fill(&mut nonce_bytes)
        .map_err(|err| anyhow::anyhow!("failed generating encryption nonce: {err}"))?;
    let nonce = aes_gcm_siv::Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(
            nonce,
            aes_gcm_siv::aead::Payload {
                msg: plaintext,
                aad: AAD,
            },
        )
        .map_err(|_| anyhow::anyhow!("failed encrypting local state"))?;

    let blob = EncryptedBlob {
        // Versioned envelope enables format evolution without silent misreads.
        v: 1,
        nonce: B64.encode(nonce_bytes),
        ciphertext: B64.encode(ciphertext),
    };

    serde_json::to_vec(&blob).context("failed serializing encrypted blob")
}

/// Decrypt persisted bytes.
pub fn decrypt(payload: &[u8]) -> Result<Vec<u8>> {
    let blob: EncryptedBlob =
        serde_json::from_slice(payload).context("payload is not encrypted blob json")?;
    if blob.v != 1 {
        return Err(anyhow::anyhow!("unsupported encrypted blob version"));
    }

    let key = load_or_create_key()?;
    let cipher = aes_gcm_siv::Aes256GcmSiv::new_from_slice(&key)
        .map_err(|_| anyhow::anyhow!("invalid AES key length"))?;

    let nonce_raw = B64.decode(blob.nonce).context("invalid encrypted nonce")?;
    if nonce_raw.len() != 12 {
        return Err(anyhow::anyhow!("invalid nonce length"));
    }
    let nonce = aes_gcm_siv::Nonce::from_slice(&nonce_raw);

    let ciphertext = B64
        .decode(blob.ciphertext)
        .context("invalid encrypted ciphertext")?;

    cipher
        .decrypt(
            nonce,
            aes_gcm_siv::aead::Payload {
                msg: &ciphertext,
                aad: AAD,
            },
        )
        .map_err(|_| anyhow::anyhow!("failed decrypting local state"))
}

fn load_or_create_key() -> Result<[u8; 32]> {
    let entry =
        Entry::new(SERVICE_NAME, STATE_KEY_ID).context("failed creating state key entry")?;

    if let Ok(secret) = entry.get_password() {
        let raw = B64
            .decode(secret)
            .context("state key in keyring is invalid base64")?;
        let arr: [u8; 32] = raw
            .try_into()
            .map_err(|_| anyhow::anyhow!("state key has invalid length"))?;
        return Ok(arr);
    }

    let mut key = [0_u8; 32];
    getrandom::fill(&mut key)
        .map_err(|err| anyhow::anyhow!("failed generating state key: {err}"))?;
    entry
        .set_password(&B64.encode(key))
        .context("failed storing state key in keyring")?;
    Ok(key)
}
