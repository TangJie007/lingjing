//! API Key 加密存储模块 (API-001)
//!
//! 使用 ring AEAD (AES-256-GCM) 对 API Key 进行本地加密存储。
//! 加密密钥存储在 `{app_data_dir}/.crypto_key`，密文存储在 `{app_data_dir}/api_keys.enc`。
//!
//! 设计要点：
//! - 首次使用时生成随机 256-bit 密钥，持久化到本地文件
//! - 每次加密生成随机 96-bit nonce，与密文拼接存储
//! - 支持多平台 Key 的独立加密存储
//! - 前端绝不接触明文 Key 的持久化存储

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

const KEY_FILE: &str = ".crypto_key";
const DATA_FILE: &str = "api_keys.enc";
const NONCE_LEN: usize = 12; // 96-bit nonce for AES-256-GCM
const KEY_LEN: usize = 32; // 256-bit key

/// 加密存储的 API Key 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedEntry {
    /// 平台标识 (volcano, deepseek, zhipu, kling)
    platform: String,
    /// Base64 编码的密文（含 nonce 前缀）
    ciphertext: String,
}

/// 加密数据文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct KeyStore {
    keys: HashMap<String, EncryptedEntry>,
}

/// 获取数据目录（与 wallpaper_engine 保持一致）
fn data_dir(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// 生成随机 nonce（12 字节）
fn random_nonce() -> Result<[u8; NONCE_LEN], String> {
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| format!("Failed to generate nonce: {}", e))?;
    Ok(nonce_bytes)
}

/// 生成随机 AES-256 密钥（32 字节）
fn random_key() -> Result<[u8; KEY_LEN], String> {
    let rng = SystemRandom::new();
    let mut key_bytes = [0u8; KEY_LEN];
    rng.fill(&mut key_bytes)
        .map_err(|e| format!("Failed to generate key: {}", e))?;
    Ok(key_bytes)
}

/// 加载或生成加密密钥
///
/// 优先从 `{app_data_dir}/.crypto_key` 读取，不存在时生成新密钥并持久化。
fn load_or_create_key(data_dir: &PathBuf) -> Result<[u8; KEY_LEN], String> {
    let key_path = data_dir.join(KEY_FILE);

    if key_path.exists() {
        let bytes = fs::read(&key_path)
            .map_err(|e| format!("Failed to read crypto key: {}", e))?;
        if bytes.len() != KEY_LEN {
            return Err("Crypto key file corrupted (wrong length)".into());
        }
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(&bytes);
        Ok(key)
    } else {
        let key = random_key()?;
        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create data dir: {}", e))?;
        }
        fs::write(&key_path, &key)
            .map_err(|e| format!("Failed to write crypto key: {}", e))?;
        Ok(key)
    }
}

/// 加载密文存储文件
fn load_keystore(data_dir: &PathBuf) -> KeyStore {
    let path = data_dir.join(DATA_FILE);
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => KeyStore::default(),
        }
    } else {
        KeyStore::default()
    }
}

/// 保存密文存储文件
fn save_keystore(data_dir: &PathBuf, store: &KeyStore) -> Result<(), String> {
    let path = data_dir.join(DATA_FILE);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create data dir: {}", e))?;
    }
    let json = serde_json::to_string(store)
        .map_err(|e| format!("Failed to serialize keystore: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write keystore: {}", e))?;
    Ok(())
}

/// 使用 AES-256-GCM 加密明文
///
/// 返回 Base64 编码的 nonce + 密文（12 字节 nonce + 密文）。
fn encrypt_bytes(key_bytes: &[u8; KEY_LEN], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let unbound_key =
        UnboundKey::new(&AES_256_GCM, key_bytes).map_err(|e| format!("Invalid key: {}", e))?;
    let key = LessSafeKey::new(unbound_key);

    let nonce_bytes = random_nonce()?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // 前缀 nonce + 密文（含 16 字节 tag）
    let mut result = Vec::with_capacity(NONCE_LEN + in_out.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);
    Ok(result)
}

/// 使用 AES-256-GCM 解密密文
///
/// `data` 格式：前 12 字节 nonce + 密文（含 16 字节 tag）
fn decrypt_bytes(key_bytes: &[u8; KEY_LEN], data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < NONCE_LEN + 16 {
        return Err("Ciphertext too short".into());
    }

    let unbound_key =
        UnboundKey::new(&AES_256_GCM, key_bytes).map_err(|e| format!("Invalid key: {}", e))?;
    let key = LessSafeKey::new(unbound_key);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&data[..NONCE_LEN]);
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = data[NONCE_LEN..].to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| "Decryption failed — wrong key or corrupted data".to_string())?;

    Ok(plaintext.to_vec())
}

/// 加密并持久化 API Key
///
/// # Arguments
/// * `app` - Tauri AppHandle，用于获取数据目录
/// * `platform` - 平台标识，如 "volcano"
/// * `plaintext` - 明文的 API Key
///
/// # Returns
/// 掩码后的 Key（前 3 + **** + 后 4）
#[tauri::command]
pub fn crypto_encrypt(
    app: tauri::AppHandle,
    platform: String,
    plaintext: String,
) -> Result<String, String> {
    if plaintext.is_empty() {
        return Err("API Key cannot be empty".into());
    }

    let dir = data_dir(&app);
    let key_bytes = load_or_create_key(&dir)?;
    let cipher_bytes = encrypt_bytes(&key_bytes, plaintext.as_bytes())?;
    let ciphertext = base64_encode(&cipher_bytes);

    let mut store = load_keystore(&dir);
    store.keys.insert(
        platform.clone(),
        EncryptedEntry {
            platform: platform.clone(),
            ciphertext,
        },
    );
    save_keystore(&dir, &store)?;

    Ok(mask_key(&plaintext))
}

/// 解密读取 API Key
///
/// # Arguments
/// * `app` - Tauri AppHandle
/// * `platform` - 平台标识
///
/// # Returns
/// 明文 API Key，如果该平台未配置则返回空字符串
#[tauri::command]
pub fn crypto_decrypt(app: tauri::AppHandle, platform: String) -> Result<String, String> {
    let dir = data_dir(&app);
    let key_bytes = load_or_create_key(&dir)?;
    let store = load_keystore(&dir);

    match store.keys.get(&platform) {
        Some(entry) => {
            let data = base64_decode(&entry.ciphertext)?;
            let plaintext = decrypt_bytes(&key_bytes, &data)?;
            String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
        }
        None => Ok(String::new()), // 未配置，返回空
    }
}

/// 获取所有已配置平台的掩码 Key 列表
///
/// 返回 `[{ platform, maskedKey }]`，供前端初始化时加载。
#[tauri::command]
pub fn crypto_list_platforms(app: tauri::AppHandle) -> Result<Vec<PlatformInfo>, String> {
    let dir = data_dir(&app);
    let key_bytes = load_or_create_key(&dir)?;
    let store = load_keystore(&dir);

    let mut result = Vec::new();
    for entry in store.keys.values() {
        let data = base64_decode(&entry.ciphertext)?;
        let plaintext = decrypt_bytes(&key_bytes, &data)?;
        let key_str =
            String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))?;
        result.push(PlatformInfo {
            platform: entry.platform.clone(),
            masked_key: mask_key(&key_str),
        });
    }
    Ok(result)
}

/// 平台信息（返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub platform: String,
    pub masked_key: String,
}

/// 掩码显示：保留前 3 + **** + 后 4 字符
fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &key[..3], &key[key.len() - 4..])
}

/// Base64 编码（标准 alphabet，无 padding）
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        }
    }
    result
}

/// Base64 解码
fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    const DECODE: [i8; 128] = {
        let mut table = [-1i8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < chars.len() {
            table[chars[i] as usize] = i as i8;
            i += 1;
        }
        table
    };

    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits = 0u32;

    for &b in bytes {
        let val = DECODE.get(b as usize).copied().unwrap_or(-1);
        if val < 0 {
            continue; // skip invalid chars
        }
        buffer = (buffer << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_key_normal() {
        assert_eq!(mask_key("sk-1234567890abcdef"), "sk-****cdef");
    }

    #[test]
    fn test_mask_key_short() {
        assert_eq!(mask_key("short"), "****");
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"hello world test data for encryption";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = random_key().unwrap();
        let plaintext = b"sk-test-api-key-12345";
        let cipher = encrypt_bytes(&key, plaintext).unwrap();
        let decrypted = decrypt_bytes(&key, &cipher).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key1 = random_key().unwrap();
        let key2 = random_key().unwrap();
        let cipher = encrypt_bytes(&key1, b"test").unwrap();
        assert!(decrypt_bytes(&key2, &cipher).is_err());
    }

    #[test]
    fn test_decrypt_corrupted_data_fails() {
        let key = random_key().unwrap();
        let mut cipher = encrypt_bytes(&key, b"test").unwrap();
        if cipher.len() > 13 {
            cipher[13] ^= 0xFF;
        }
        assert!(decrypt_bytes(&key, &cipher).is_err());
    }
}
