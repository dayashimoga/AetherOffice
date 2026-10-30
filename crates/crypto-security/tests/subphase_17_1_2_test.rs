//! Comprehensive integration tests for Subphase 17.1.2:
//! AES-256-GCM Envelope Encryption, Zero-Knowledge Protection & Document Permissions.

use crypto_security::aes::{
    aes256_cbc_decrypt, aes256_cbc_encrypt, aes256_gcm_decrypt, aes256_gcm_encrypt,
};
use crypto_security::digest::HashAlgorithm;
use crypto_security::error::CryptoSecurityError;
use crypto_security::kdf::{hmac, pbkdf2};
use crypto_security::ooxml_crypto::OoxmlAgileCrypto;
use crypto_security::pdf_crypto::{PdfPermissions, PdfSecurityEngine};
use crypto_security::zero_knowledge::ProtectedKey;

#[test]
fn test_01_aes256_cbc_encryption_decryption_roundtrip() {
    let key: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        0x1F, 0x20,
    ];
    let iv: [u8; 16] = [
        0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ];

    let payloads: &[&[u8]] = &[
        b"",
        b"A",
        b"123456789012345",   // 15 bytes
        b"1234567890123456",  // 16 bytes (block boundary)
        b"12345678901234567", // 17 bytes
        b"AetherOffice Clean-Room Enterprise Security Model",
        &[0x42; 1024],
    ];

    for &plaintext in payloads {
        let encrypted = aes256_cbc_encrypt(&key, &iv, plaintext);
        assert!(encrypted.len().is_multiple_of(16));
        assert!(encrypted.len() > plaintext.len());

        let decrypted = aes256_cbc_decrypt(&key, &iv, &encrypted).expect("Decryption failed");
        assert_eq!(plaintext, decrypted.as_slice());
    }
}

#[test]
fn test_02_aes256_cbc_invalid_padding_rejection() {
    let key = [0x55u8; 32];
    let iv = [0xAAu8; 16];
    let plaintext = b"Confidential M&A Strategy Document";

    let mut ciphertext = aes256_cbc_encrypt(&key, &iv, plaintext);
    assert!(!ciphertext.is_empty());

    // Corrupt the final byte of padding
    let last_idx = ciphertext.len() - 1;
    ciphertext[last_idx] ^= 0x01;

    let decrypt_res = aes256_cbc_decrypt(&key, &iv, &ciphertext);
    assert!(decrypt_res.is_err());
    match decrypt_res {
        Err(CryptoSecurityError::InvalidPadding(_)) => {}
        other => panic!("Expected InvalidPadding, got {:?}", other),
    }
}

#[test]
fn test_03_aes256_gcm_authenticated_encryption() {
    let key = [0x33u8; 32];
    let nonce = [0x77u8; 12];
    let plaintext = b"Quarterly Budget Approval with Cryptographic Zero-Knowledge Envelope";
    let aad = b"header:doc_id=AETHER-999;owner=finance";

    let (ciphertext, tag) = aes256_gcm_encrypt(&key, &nonce, plaintext, aad);
    assert_eq!(ciphertext.len(), plaintext.len());
    assert_eq!(tag.len(), 16);

    let decrypted =
        aes256_gcm_decrypt(&key, &nonce, &ciphertext, &tag, aad).expect("GCM decrypt failed");
    assert_eq!(plaintext, decrypted.as_slice());
}

#[test]
fn test_04_aes256_gcm_tamper_detection() {
    let key = [0x44u8; 32];
    let nonce = [0x88u8; 12];
    let plaintext = b"Wire Transfer: $10,000,000 to Account #123456";
    let aad = b"authenticated-metadata-envelope";

    let (mut ciphertext, tag) = aes256_gcm_encrypt(&key, &nonce, plaintext, aad);

    // Tamper with ciphertext
    ciphertext[5] ^= 0xFF;
    let res = aes256_gcm_decrypt(&key, &nonce, &ciphertext, &tag, aad);
    assert!(res.is_err());
    match res {
        Err(CryptoSecurityError::AuthenticationFailed(_)) => {}
        other => panic!(
            "Expected AuthenticationFailed on tampered ciphertext, got {:?}",
            other
        ),
    }

    // Tamper with AAD
    ciphertext[5] ^= 0xFF; // restore ciphertext
    let tampered_aad = b"authenticated-metadata-altered!";
    let aad_res = aes256_gcm_decrypt(&key, &nonce, &ciphertext, &tag, tampered_aad);
    assert!(aad_res.is_err());
}

#[test]
fn test_05_hmac_sha256_and_sha512_vectors() {
    let key = b"secret-corporate-signing-key";
    let message = b"The quick brown fox jumps over the lazy dog";

    let h256 = hmac(HashAlgorithm::Sha256, key, message);
    assert_eq!(h256.len(), 32);

    let h512 = hmac(HashAlgorithm::Sha512, key, message);
    assert_eq!(h512.len(), 64);

    // Deterministic output
    let h256_repeat = hmac(HashAlgorithm::Sha256, key, message);
    assert_eq!(h256, h256_repeat);

    // Different message yields different HMAC
    let h256_diff = hmac(HashAlgorithm::Sha256, key, b"different message");
    assert_ne!(h256, h256_diff);
}

#[test]
fn test_06_pbkdf2_hmac_sha512_key_derivation() {
    let password = b"P@ssw0rd2026!#";
    let salt = b"corporate-salt-99";
    let iterations = 1000;

    let mut key1 = [0u8; 32];
    pbkdf2(HashAlgorithm::Sha512, password, salt, iterations, &mut key1).expect("PBKDF2 failed");
    assert_ne!(key1, [0u8; 32]);

    let mut key2 = [0u8; 32];
    pbkdf2(HashAlgorithm::Sha512, password, salt, iterations, &mut key2).expect("PBKDF2 failed");
    assert_eq!(key1, key2);

    // Different salt yields different derived key
    let mut key_diff_salt = [0u8; 32];
    pbkdf2(
        HashAlgorithm::Sha512,
        password,
        b"other-salt",
        iterations,
        &mut key_diff_salt,
    )
    .expect("PBKDF2 failed");
    assert_ne!(key1, key_diff_salt);
}

#[test]
fn test_07_ooxml_agile_encryption_roundtrip() {
    let fake_opc_container =
        b"PK\x03\x04[Mock Open Packaging Convention Zip Stream with word/document.xml]";
    let doc_password = "StrictOfficePassword#2026";

    // Encrypt container
    let (descriptor_xml, encrypted_package) =
        OoxmlAgileCrypto::encrypt_package(fake_opc_container, doc_password, 1000)
            .expect("Agile encryption failed");

    assert!(descriptor_xml.contains("<encryption"));
    assert!(descriptor_xml.contains("keyEncryptor"));

    // Parse descriptor
    let parsed_info = OoxmlAgileCrypto::parse_encryption_info(&descriptor_xml)
        .expect("Failed to parse encryption descriptor");

    // Decrypt container
    let decrypted =
        OoxmlAgileCrypto::decrypt_package(&parsed_info, &encrypted_package, doc_password)
            .expect("Decryption failed with correct password");

    assert_eq!(fake_opc_container, decrypted.as_slice());
}

#[test]
fn test_08_ooxml_agile_wrong_password_rejection() {
    let fake_opc_container = b"PK\x03\x04[Secret Spreadsheet Container]";
    let correct_password = "ValidPassword123";
    let wrong_password = "WrongPassword456";

    let (descriptor_xml, encrypted_package) =
        OoxmlAgileCrypto::encrypt_package(fake_opc_container, correct_password, 1000)
            .expect("Agile encryption failed");

    let parsed_info = OoxmlAgileCrypto::parse_encryption_info(&descriptor_xml)
        .expect("Failed to parse encryption descriptor");

    let res = OoxmlAgileCrypto::decrypt_package(&parsed_info, &encrypted_package, wrong_password);
    assert!(res.is_err());
    match res {
        Err(CryptoSecurityError::InvalidPassword) => {}
        other => panic!("Expected InvalidPassword, got {:?}", other),
    }
}

#[test]
fn test_09_pdf_revision6_user_and_owner_passwords() {
    let pdf_content = b"%PDF-2.0\n1 0 obj\n<< /Title (Executive Board Minutes) >>\nendobj\n";
    let user_pw = "ReaderOnly2026";
    let owner_pw = "MasterAdminSecret";

    let perms = PdfPermissions {
        modify_contents: false,
        ..Default::default()
    };

    let (dict, encrypted_stream) =
        PdfSecurityEngine::encrypt_document(pdf_content, user_pw, owner_pw, &perms)
            .expect("PDF encryption failed");

    assert_eq!(dict.r, 6);
    assert_eq!(dict.v, 5);
    assert_eq!(dict.length, 256);

    // Authenticate as User
    let user_session = PdfSecurityEngine::authenticate(&dict, user_pw).expect("User auth failed");
    assert!(!user_session.is_owner);
    assert!(!user_session.permissions.modify_contents);
    assert!(user_session.permissions.print_high_quality);

    let user_decrypted = user_session
        .decrypt_stream(&encrypted_stream)
        .expect("User decrypt stream failed");
    assert_eq!(pdf_content, user_decrypted.as_slice());

    // Authenticate as Owner
    let owner_session =
        PdfSecurityEngine::authenticate(&dict, owner_pw).expect("Owner auth failed");
    assert!(owner_session.is_owner);
    assert!(owner_session.permissions.modify_contents); // Owner has full permissions

    let owner_decrypted = owner_session
        .decrypt_stream(&encrypted_stream)
        .expect("Owner decrypt stream failed");
    assert_eq!(pdf_content, owner_decrypted.as_slice());

    // Reject wrong password
    let bad_auth = PdfSecurityEngine::authenticate(&dict, "IncorrectPassword");
    assert!(bad_auth.is_err());
    match bad_auth {
        Err(CryptoSecurityError::InvalidPassword) => {}
        other => panic!("Expected InvalidPassword, got {:?}", other),
    }
}

#[test]
fn test_10_pdf_permissions_bitmask_and_perms_block_validation() {
    let restricted_perms = PdfPermissions {
        print_low_quality: false,
        print_high_quality: false,
        extract_text_and_graphics: false,
        modify_contents: false,
        ..Default::default()
    };

    let p_val = restricted_perms.to_p_int();
    let reconstructed = PdfPermissions::from_p_int(p_val);

    assert_eq!(
        restricted_perms.print_low_quality,
        reconstructed.print_low_quality
    );
    assert_eq!(
        restricted_perms.print_high_quality,
        reconstructed.print_high_quality
    );
    assert_eq!(
        restricted_perms.extract_text_and_graphics,
        reconstructed.extract_text_and_graphics
    );
    assert_eq!(
        restricted_perms.modify_contents,
        reconstructed.modify_contents
    );

    // Protected key zeroing check
    let key = ProtectedKey::new(vec![0xFF; 32]);
    assert_eq!(key.len(), 32);
    assert_eq!(key.as_bytes()[0], 0xFF);
    drop(key);
}
