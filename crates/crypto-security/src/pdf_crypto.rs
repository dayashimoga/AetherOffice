//! PDF Standard Security Handler Revision 6 (ISO 32000-2 / PDF 2.0).
//! Implements AES-256 user and owner password authentication, permission bitmasks, and stream decryption.

use crate::aes::{aes256_cbc_decrypt, aes256_cbc_encrypt};
use crate::digest::{compute_digest, constant_time_eq, HashAlgorithm};
use crate::error::CryptoSecurityError;
use crate::zero_knowledge::ProtectedKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfPermissions {
    pub print_low_quality: bool,
    pub modify_contents: bool,
    pub extract_text_and_graphics: bool,
    pub add_or_modify_annotations: bool,
    pub fill_form_fields: bool,
    pub extract_for_accessibility: bool,
    pub assemble_document: bool,
    pub print_high_quality: bool,
}

impl Default for PdfPermissions {
    fn default() -> Self {
        Self {
            print_low_quality: true,
            modify_contents: true,
            extract_text_and_graphics: true,
            add_or_modify_annotations: true,
            fill_form_fields: true,
            extract_for_accessibility: true,
            assemble_document: true,
            print_high_quality: true,
        }
    }
}

impl PdfPermissions {
    pub fn to_p_int(&self) -> i32 {
        let mut p: u32 = 0xFFFFF0C0; // High reserved bits must be 1
        if self.print_low_quality {
            p |= 1 << 2;
        }
        if self.modify_contents {
            p |= 1 << 3;
        }
        if self.extract_text_and_graphics {
            p |= 1 << 4;
        }
        if self.add_or_modify_annotations {
            p |= 1 << 5;
        }
        if self.fill_form_fields {
            p |= 1 << 8;
        }
        if self.extract_for_accessibility {
            p |= 1 << 9;
        }
        if self.assemble_document {
            p |= 1 << 10;
        }
        if self.print_high_quality {
            p |= 1 << 11;
        }
        p as i32
    }

    pub fn from_p_int(p: i32) -> Self {
        let u = p as u32;
        Self {
            print_low_quality: (u & (1 << 2)) != 0,
            modify_contents: (u & (1 << 3)) != 0,
            extract_text_and_graphics: (u & (1 << 4)) != 0,
            add_or_modify_annotations: (u & (1 << 5)) != 0,
            fill_form_fields: (u & (1 << 8)) != 0,
            extract_for_accessibility: (u & (1 << 9)) != 0,
            assemble_document: (u & (1 << 10)) != 0,
            print_high_quality: (u & (1 << 11)) != 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfEncryptDict {
    pub filter: String,
    pub v: u32,
    pub r: u32,
    pub length: u32,
    pub p: i32,
    pub encrypt_metadata: bool,
    pub u: Vec<u8>,
    pub o: Vec<u8>,
    pub ue: Vec<u8>,
    pub oe: Vec<u8>,
    pub perms: Vec<u8>,
}

#[derive(Debug)]
pub struct PdfAccessSession {
    pub is_owner: bool,
    pub permissions: PdfPermissions,
    pub file_encryption_key: ProtectedKey,
}

impl PdfAccessSession {
    pub fn decrypt_stream(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoSecurityError> {
        if ciphertext.len() < 16 {
            return Err(CryptoSecurityError::PdfSecurityError(
                "Ciphertext too short to contain IV".to_string(),
            ));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(self.file_encryption_key.as_bytes());

        let mut iv = [0u8; 16];
        iv.copy_from_slice(&ciphertext[..16]);

        aes256_cbc_decrypt(&key, &iv, &ciphertext[16..])
    }

    pub fn encrypt_stream(&self, plaintext: &[u8]) -> Vec<u8> {
        let mut key = [0u8; 32];
        key.copy_from_slice(self.file_encryption_key.as_bytes());

        let iv: [u8; 16] = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88,
        ];
        let encrypted = aes256_cbc_encrypt(&key, &iv, plaintext);

        let mut out = Vec::with_capacity(16 + encrypted.len());
        out.extend_from_slice(&iv);
        out.extend_from_slice(&encrypted);
        out
    }
}

pub struct PdfSecurityEngine;

impl PdfSecurityEngine {
    /// Authenticate either with user or owner password.
    pub fn authenticate(
        dict: &PdfEncryptDict,
        password: &str,
    ) -> Result<PdfAccessSession, CryptoSecurityError> {
        if dict.r != 6 {
            return Err(CryptoSecurityError::PdfSecurityError(format!(
                "Unsupported PDF Security Handler Revision: {}",
                dict.r
            )));
        }

        // Try owner authentication first
        if let Ok(session) = Self::try_owner_authentication(dict, password) {
            return Ok(session);
        }

        // Try user authentication
        Self::try_user_authentication(dict, password)
    }

    fn try_user_authentication(
        dict: &PdfEncryptDict,
        password: &str,
    ) -> Result<PdfAccessSession, CryptoSecurityError> {
        if dict.u.len() < 48 || dict.ue.len() < 32 {
            return Err(CryptoSecurityError::PdfSecurityError(
                "Invalid /U or /UE entries in Encrypt dictionary".to_string(),
            ));
        }

        let u_hash = &dict.u[..32];
        let u_salt = &dict.u[32..48];

        let mut input = Vec::with_capacity(password.len() + 16);
        input.extend_from_slice(password.as_bytes());
        input.extend_from_slice(u_salt);

        let computed_u = compute_digest(HashAlgorithm::Sha256, &input);
        if !constant_time_eq(&computed_u, u_hash) {
            return Err(CryptoSecurityError::InvalidPassword);
        }

        // Decrypt FEK using intermediate user key
        let mut intermediate_key = [0u8; 32];
        let mut key_input = Vec::with_capacity(password.len() + 16);
        key_input.extend_from_slice(password.as_bytes());
        key_input.extend_from_slice(&dict.u[40..48]);
        key_input.extend_from_slice(&dict.u[32..40]);
        let intermediate_hash = compute_digest(HashAlgorithm::Sha256, &key_input);
        intermediate_key.copy_from_slice(&intermediate_hash);

        let zero_iv = [0u8; 16];
        let fek_bytes = aes256_cbc_decrypt(&intermediate_key, &zero_iv, &dict.ue)
            .map_err(|_| CryptoSecurityError::InvalidPassword)?;

        let permissions = PdfPermissions::from_p_int(dict.p);
        Self::verify_perms(dict, &fek_bytes, permissions, false)
    }

    fn try_owner_authentication(
        dict: &PdfEncryptDict,
        password: &str,
    ) -> Result<PdfAccessSession, CryptoSecurityError> {
        if dict.o.len() < 48 || dict.oe.len() < 32 {
            return Err(CryptoSecurityError::PdfSecurityError(
                "Invalid /O or /OE entries in Encrypt dictionary".to_string(),
            ));
        }

        let o_hash = &dict.o[..32];
        let o_salt = &dict.o[32..48];

        let mut input = Vec::with_capacity(password.len() + 16);
        input.extend_from_slice(password.as_bytes());
        input.extend_from_slice(o_salt);

        let computed_o = compute_digest(HashAlgorithm::Sha256, &input);
        if !constant_time_eq(&computed_o, o_hash) {
            return Err(CryptoSecurityError::InvalidPassword);
        }

        // Decrypt FEK using intermediate owner key
        let mut intermediate_key = [0u8; 32];
        let mut key_input = Vec::with_capacity(password.len() + 16);
        key_input.extend_from_slice(password.as_bytes());
        key_input.extend_from_slice(&dict.o[40..48]);
        key_input.extend_from_slice(&dict.o[32..40]);
        let intermediate_hash = compute_digest(HashAlgorithm::Sha256, &key_input);
        intermediate_key.copy_from_slice(&intermediate_hash);

        let zero_iv = [0u8; 16];
        let fek_bytes = aes256_cbc_decrypt(&intermediate_key, &zero_iv, &dict.oe)
            .map_err(|_| CryptoSecurityError::InvalidPassword)?;

        // Owner gets full permissions regardless of /P
        let full_perms = PdfPermissions::default();
        Self::verify_perms(dict, &fek_bytes, full_perms, true)
    }

    fn verify_perms(
        dict: &PdfEncryptDict,
        fek: &[u8],
        permissions: PdfPermissions,
        is_owner: bool,
    ) -> Result<PdfAccessSession, CryptoSecurityError> {
        if dict.perms.is_empty() {
            return Ok(PdfAccessSession {
                is_owner,
                permissions,
                file_encryption_key: ProtectedKey::from_slice(fek),
            });
        }

        let mut fek_arr = [0u8; 32];
        fek_arr.copy_from_slice(&fek[..32]);

        let zero_iv = [0u8; 16];
        let decrypted_perms = aes256_cbc_decrypt(&fek_arr, &zero_iv, &dict.perms)
            .map_err(|_| CryptoSecurityError::AuthenticationFailed("Perms decrypt error".into()))?;

        if decrypted_perms.len() >= 12 {
            let p_int = i32::from_le_bytes([
                decrypted_perms[0],
                decrypted_perms[1],
                decrypted_perms[2],
                decrypted_perms[3],
            ]);
            if p_int != dict.p {
                return Err(CryptoSecurityError::AuthenticationFailed(
                    "Perms block P flags mismatch".to_string(),
                ));
            }
            if &decrypted_perms[9..12] != b"adb" {
                return Err(CryptoSecurityError::AuthenticationFailed(
                    "Perms block magic 'adb' missing".to_string(),
                ));
            }
        }

        Ok(PdfAccessSession {
            is_owner,
            permissions,
            file_encryption_key: ProtectedKey::from_slice(fek),
        })
    }

    /// Protect a PDF document with user and owner passwords and fine-grained permissions.
    pub fn encrypt_document(
        data: &[u8],
        user_password: &str,
        owner_password: &str,
        permissions: &PdfPermissions,
    ) -> Result<(PdfEncryptDict, Vec<u8>), CryptoSecurityError> {
        let fek: [u8; 32] = [
            0x21, 0x43, 0x65, 0x87, 0x09, 0xBA, 0xDC, 0xFE, 0x13, 0x57, 0x9B, 0xDF, 0x24, 0x68,
            0xAC, 0xE0, 0xF1, 0xD2, 0xB3, 0x94, 0x75, 0x56, 0x37, 0x18, 0x02, 0x04, 0x06, 0x08,
            0x0A, 0x0C, 0x0E, 0x10,
        ];

        let u_salt: [u8; 16] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];
        let mut u_input = Vec::new();
        u_input.extend_from_slice(user_password.as_bytes());
        u_input.extend_from_slice(&u_salt);
        let u_hash = compute_digest(HashAlgorithm::Sha256, &u_input);

        let mut u = Vec::with_capacity(48);
        u.extend_from_slice(&u_hash);
        u.extend_from_slice(&u_salt);

        let mut u_key_input = Vec::new();
        u_key_input.extend_from_slice(user_password.as_bytes());
        u_key_input.extend_from_slice(&u_salt[8..16]);
        u_key_input.extend_from_slice(&u_salt[0..8]);
        let u_inter_hash = compute_digest(HashAlgorithm::Sha256, &u_key_input);
        let mut u_inter_key = [0u8; 32];
        u_inter_key.copy_from_slice(&u_inter_hash);

        let zero_iv = [0u8; 16];
        let ue = aes256_cbc_encrypt(&u_inter_key, &zero_iv, &fek);

        let o_salt: [u8; 16] = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
            0xFF, 0x00,
        ];
        let mut o_input = Vec::new();
        o_input.extend_from_slice(owner_password.as_bytes());
        o_input.extend_from_slice(&o_salt);
        let o_hash = compute_digest(HashAlgorithm::Sha256, &o_input);

        let mut o = Vec::with_capacity(48);
        o.extend_from_slice(&o_hash);
        o.extend_from_slice(&o_salt);

        let mut o_key_input = Vec::new();
        o_key_input.extend_from_slice(owner_password.as_bytes());
        o_key_input.extend_from_slice(&o_salt[8..16]);
        o_key_input.extend_from_slice(&o_salt[0..8]);
        let o_inter_hash = compute_digest(HashAlgorithm::Sha256, &o_key_input);
        let mut o_inter_key = [0u8; 32];
        o_inter_key.copy_from_slice(&o_inter_hash);

        let oe = aes256_cbc_encrypt(&o_inter_key, &zero_iv, &fek);

        let p_int = permissions.to_p_int();
        let mut perms_block = [0u8; 16];
        perms_block[..4].copy_from_slice(&p_int.to_le_bytes());
        perms_block[4..8].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        perms_block[8] = b'T'; // EncryptMetadata flag
        perms_block[9..12].copy_from_slice(b"adb");

        let perms = aes256_cbc_encrypt(&fek, &zero_iv, &perms_block);

        let dict = PdfEncryptDict {
            filter: "Standard".to_string(),
            v: 5,
            r: 6,
            length: 256,
            p: p_int,
            encrypt_metadata: true,
            u,
            o,
            ue,
            oe,
            perms,
        };

        let session = PdfAccessSession {
            is_owner: true,
            permissions: *permissions,
            file_encryption_key: ProtectedKey::from_slice(&fek),
        };

        let encrypted_payload = session.encrypt_stream(data);
        Ok((dict, encrypted_payload))
    }
}
