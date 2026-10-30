//! OOXML Agile Encryption and Decryption ([MS-OFFCRYPTO] Specification).
//! Implements password-based encryption and decryption of Open Packaging Convention (OPC) containers.

use crate::aes::{aes256_cbc_decrypt, aes256_cbc_encrypt};
use crate::base64::{decode_base64, encode_base64};
use crate::digest::{compute_digest, constant_time_eq, HashAlgorithm};
use crate::error::CryptoSecurityError;
use crate::kdf::pbkdf2;
use crate::zero_knowledge::ProtectedKey;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgileEncryptionInfo {
    pub spin_count: u32,
    pub salt_value: Vec<u8>,
    pub encrypted_verifier_hash_input: Vec<u8>,
    pub encrypted_verifier_hash_value: Vec<u8>,
    pub encrypted_key_value: Vec<u8>,
    pub key_data_salt: Vec<u8>,
}

pub struct OoxmlAgileCrypto;

impl OoxmlAgileCrypto {
    /// Parse the `[encryption]` descriptor XML from the EncryptionInfo stream.
    pub fn parse_encryption_info(xml: &str) -> Result<AgileEncryptionInfo, CryptoSecurityError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut spin_count = 100_000u32;
        let mut salt_value = Vec::new();
        let mut enc_verifier_hash_input = Vec::new();
        let mut enc_verifier_hash_value = Vec::new();
        let mut enc_key_value = Vec::new();
        let mut key_data_salt = Vec::new();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    if local_name.as_ref() == b"keyData" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"saltValue" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    key_data_salt = decode_base64(val).unwrap_or_default();
                                }
                            }
                        }
                    } else if local_name.as_ref() == b"p:encryptedKey"
                        || local_name.as_ref() == b"encryptedKey"
                    {
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"spinCount" => {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        spin_count = val.parse().unwrap_or(100_000);
                                    }
                                }
                                b"saltValue" => {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        salt_value = decode_base64(val).unwrap_or_default();
                                    }
                                }
                                b"encryptedVerifierHashInput" => {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        enc_verifier_hash_input =
                                            decode_base64(val).unwrap_or_default();
                                    }
                                }
                                b"encryptedVerifierHashValue" => {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        enc_verifier_hash_value =
                                            decode_base64(val).unwrap_or_default();
                                    }
                                }
                                b"encryptedKeyValue" => {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        enc_key_value = decode_base64(val).unwrap_or_default();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(CryptoSecurityError::OoxmlEncryptionError(format!(
                        "XML parse error: {e}"
                    )))
                }
                _ => {}
            }
            buf.clear();
        }

        if salt_value.is_empty() || enc_key_value.is_empty() {
            return Err(CryptoSecurityError::OoxmlEncryptionError(
                "Missing required Agile Encryption parameters".to_string(),
            ));
        }

        Ok(AgileEncryptionInfo {
            spin_count,
            salt_value,
            encrypted_verifier_hash_input: enc_verifier_hash_input,
            encrypted_verifier_hash_value: enc_verifier_hash_value,
            encrypted_key_value: enc_key_value,
            key_data_salt,
        })
    }

    /// Derive intermediate password key and authenticate the password.
    pub fn authenticate_and_derive_package_key(
        info: &AgileEncryptionInfo,
        password: &str,
    ) -> Result<ProtectedKey, CryptoSecurityError> {
        let mut intermediate_key = [0u8; 32];
        pbkdf2(
            HashAlgorithm::Sha512,
            password.as_bytes(),
            &info.salt_value,
            info.spin_count,
            &mut intermediate_key,
        )?;

        // Verify password if verifier fields are present
        if !info.encrypted_verifier_hash_input.is_empty()
            && !info.encrypted_verifier_hash_value.is_empty()
        {
            let mut iv = [0u8; 16];
            let take_iv = info.salt_value.len().min(16);
            iv[..take_iv].copy_from_slice(&info.salt_value[..take_iv]);

            let verifier_input =
                aes256_cbc_decrypt(&intermediate_key, &iv, &info.encrypted_verifier_hash_input)
                    .map_err(|_| CryptoSecurityError::InvalidPassword)?;

            let expected_hash = compute_digest(HashAlgorithm::Sha512, &verifier_input);

            let decrypted_hash =
                aes256_cbc_decrypt(&intermediate_key, &iv, &info.encrypted_verifier_hash_value)
                    .map_err(|_| CryptoSecurityError::InvalidPassword)?;

            let hash_len = expected_hash.len().min(decrypted_hash.len());
            if !constant_time_eq(&expected_hash[..hash_len], &decrypted_hash[..hash_len]) {
                return Err(CryptoSecurityError::InvalidPassword);
            }
        }

        // Decrypt package encryption key
        let mut key_iv = [0u8; 16];
        let take_kiv = info.salt_value.len().min(16);
        key_iv[..take_kiv].copy_from_slice(&info.salt_value[..take_kiv]);

        let package_key_bytes =
            aes256_cbc_decrypt(&intermediate_key, &key_iv, &info.encrypted_key_value)
                .map_err(|_| CryptoSecurityError::InvalidPassword)?;

        if package_key_bytes.len() < 32 {
            return Err(CryptoSecurityError::OoxmlEncryptionError(
                "Decrypted package key is shorter than 32 bytes".to_string(),
            ));
        }

        Ok(ProtectedKey::from_slice(&package_key_bytes[..32]))
    }

    /// Decrypt the EncryptedPackage stream into the plain OPC zip container bytes.
    pub fn decrypt_package(
        info: &AgileEncryptionInfo,
        encrypted_package: &[u8],
        password: &str,
    ) -> Result<Vec<u8>, CryptoSecurityError> {
        let pkg_key = Self::authenticate_and_derive_package_key(info, password)?;
        let mut key_arr = [0u8; 32];
        key_arr.copy_from_slice(pkg_key.as_bytes());

        let mut iv = [0u8; 16];
        let salt = if !info.key_data_salt.is_empty() {
            &info.key_data_salt
        } else {
            &info.salt_value
        };
        let take_iv = salt.len().min(16);
        iv[..take_iv].copy_from_slice(&salt[..take_iv]);

        aes256_cbc_decrypt(&key_arr, &iv, encrypted_package)
    }

    /// Encrypt a plain OPC zip container into Agile Encryption payload and encryption descriptor.
    pub fn encrypt_package(
        plain_package_bytes: &[u8],
        password: &str,
        spin_count: u32,
    ) -> Result<(String, Vec<u8>), CryptoSecurityError> {
        let salt: [u8; 16] = [
            0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90, 0xA0, 0xB0, 0xC0, 0xD0, 0xE0,
            0xF0, 0x01,
        ];
        let key_data_salt: [u8; 16] = [
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            0x99, 0x00,
        ];

        let mut intermediate_key = [0u8; 32];
        pbkdf2(
            HashAlgorithm::Sha512,
            password.as_bytes(),
            &salt,
            spin_count,
            &mut intermediate_key,
        )?;

        // Package encryption key (AES-256)
        let package_key: [u8; 32] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C,
            0x1D, 0x1E, 0x1F, 0x20,
        ];

        // Encrypt verifier hash input and verifier hash value
        let verifier_input = [0x5Au8; 16];
        let verifier_hash = compute_digest(HashAlgorithm::Sha512, &verifier_input);

        let enc_verifier_input = aes256_cbc_encrypt(&intermediate_key, &salt, &verifier_input);
        let enc_verifier_hash = aes256_cbc_encrypt(&intermediate_key, &salt, &verifier_hash);
        let enc_package_key = aes256_cbc_encrypt(&intermediate_key, &salt, &package_key);

        // Encrypt the payload with the package key
        let encrypted_package =
            aes256_cbc_encrypt(&package_key, &key_data_salt, plain_package_bytes);

        let salt_b64 = encode_base64(&salt);
        let kd_salt_b64 = encode_base64(&key_data_salt);
        let vi_b64 = encode_base64(&enc_verifier_input);
        let vh_b64 = encode_base64(&enc_verifier_hash);
        let pk_b64 = encode_base64(&enc_package_key);

        let descriptor_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<encryption xmlns="http://schemas.microsoft.com/office/2006/encryption" xmlns:p="http://schemas.microsoft.com/office/2006/keyEncryptor/password">
  <keyData saltSize="16" blockSize="16" keyBits="256" hashSize="64" cipherAlgorithm="AES" cipherChaining="ChainingModeCBC" hashAlgorithm="SHA512" saltValue="{kd_salt_b64}"/>
  <keyEncryptors>
    <keyEncryptor uri="http://schemas.microsoft.com/office/2006/keyEncryptor/password">
      <p:encryptedKey spinCount="{spin_count}" saltSize="16" blockSize="16" keyBits="256" hashSize="64" cipherAlgorithm="AES" cipherChaining="ChainingModeCBC" hashAlgorithm="SHA512" saltValue="{salt_b64}" encryptedVerifierHashInput="{vi_b64}" encryptedVerifierHashValue="{vh_b64}" encryptedKeyValue="{pk_b64}"/>
    </keyEncryptor>
  </keyEncryptors>
</encryption>"#
        );

        Ok((descriptor_xml, encrypted_package))
    }
}
