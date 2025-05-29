use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::console;
use pgp::types::{KeyFlags, KeyTrait, PublicKeyTrait, SecretKeyTrait, UserID, Password, CompressionAlgorithm, PublicKeyAlgorithmSet};
use pgp::crypto::{sym::SymmetricKeyAlgorithm, pubkey::PublicKeyAlgorithm, hash::HashAlgorithm};
use pgp::composed::{SignedSecretKey, SignedPublicKey, Message, Deserializable, SignedUser, KeyDetails, KeyVersion};
use pgp::armor::{Dearmor, ArmorOptions, ArmorType};
use pgp::packet::SignatureType;
use pgp::errors::Result as PgpResult; // To avoid conflict with std::result::Result
use pgp::ser::Serialize as PgpSerialize; // To avoid conflict with serde::Serialize

use std::io::{Cursor, Read, Write}; // For armor/dearmor
use std::time::{SystemTime, Duration}; // For key creation time
use std::str::FromStr; // For UserID parsing

// Structs for parameters and results (already defined, ensure they are compatible)
// KeyGenerationParams, KeyPairResult, EncryptParams, DecryptParams, SignParams, VerifyParams, VerifyResult

#[derive(Serialize, Deserialize)]
pub struct KeyGenerationParams {
    pub user_id: String,
    pub passphrase: String,
}

#[derive(Serialize, Deserialize)]
pub struct KeyPairResult {
    pub key_id: String,
    pub fingerprint: String,
    pub user_id: String,
    pub algorithm: String,
    pub public_key_armored: String,
    pub private_key_armored: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct EncryptParams {
    pub recipient_public_keys: Vec<String>, // Changed from recipient_key_ids
    pub plaintext: String,
    // Optional: pub signing_secret_key_armored: Option<String>, // For sign-then-encrypt
    // Optional: pub signing_secret_key_passphrase: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DecryptParams {
    // pub private_key_id: String, // Will use armored key directly
    pub armored_private_key: String,
    pub passphrase: String,
    pub ciphertext: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignParams {
    // pub private_key_id: String, // Will use armored key directly
    pub armored_private_key: String,
    pub passphrase: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyParams {
    // pub signer_key_id: String, // Will use armored key directly
    pub armored_public_key: String,
    pub message: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyResult {
    pub is_valid: bool,
    pub message: String,
}

// Utility function to log to browser console
fn log(s: &str) {
    console::log_1(&s.into());
}

// Helper to convert PGP errors to JsValue
fn pgp_error_to_jsvalue(e: pgp::errors::Error) -> JsValue {
    JsValue::from_str(&format!("PGP Error: {}", e))
}


#[wasm_bindgen]
pub fn generate_key_pair(params_json: &str) -> Result<String, JsValue> {
    log("🔑 Generating post-quantum key pair with pgp crate...");
    
    let params: KeyGenerationParams = serde_json::from_str(params_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid parameters: {}", e)))?;

    // Key preferences
    let mut primary_key_params = KeyDetails::new(
        KeyVersion::V6, // PQC keys are V6
        PublicKeyAlgorithm::Dilithium5, // Primary key for signing
        120, // bits - placeholder, actual strength defined by algorithm
        SystemTime::now(),
        Duration::from_secs(0), // No expiration
    );
    primary_key_params.set_preferred_hash_algorithms(vec![HashAlgorithm::SHA512, HashAlgorithm::SHA256]);
    primary_key_params.set_preferred_symmetric_algorithms(vec![SymmetricKeyAlgorithm::AES256, SymmetricKeyAlgorithm::AES128]);
    primary_key_params.set_preferred_compression_algorithms(vec![CompressionAlgorithm::ZLIB, CompressionAlgorithm::Uncompressed]);


    let mut subkey_params = KeyDetails::new(
        KeyVersion::V6,
        PublicKeyAlgorithm::Kyber1024, // Subkey for encryption
        120, // bits - placeholder
        SystemTime::now(),
        Duration::from_secs(0), // No expiration
    );
    subkey_params.set_preferred_hash_algorithms(vec![HashAlgorithm::SHA512, HashAlgorithm::SHA256]);
    subkey_params.set_preferred_symmetric_algorithms(vec![SymmetricKeyAlgorithm::AES256, SymmetricKeyAlgorithm::AES128]);
    subkey_params.set_preferred_compression_algorithms(vec![CompressionAlgorithm::ZLIB, CompressionAlgorithm::Uncompressed]);


    let mut secret_key = SignedSecretKey::new(primary_key_params)
        .map_err(pgp_error_to_jsvalue)?;
    
    // Add User ID
    let user_id = UserID::from_str(&params.user_id)
        .map_err(|e| JsValue::from_str(&format!("Invalid UserID format: {}",e)))?;

    let signed_user = SignedUser::new(user_id, SignatureType::PositiveCertification, HashAlgorithm::SHA512, &secret_key)
        .map_err(pgp_error_to_jsvalue)?;
    secret_key.add_user(signed_user).map_err(pgp_error_to_jsvalue)?;
    
    // Add subkey for encryption
    secret_key.add_subkey(subkey_params, SignatureType::SubkeyBinding, HashAlgorithm::SHA512)
        .map_err(pgp_error_to_jsvalue)?;

    // Protect the key with passphrase if provided
    let final_secret_key = if !params.passphrase.is_empty() {
        let pw = Password::new(&params.passphrase);
        secret_key.encrypt_with_password(SymmetricKeyAlgorithm::AES128, HashAlgorithm::SHA256, &pw)
            .map_err(pgp_error_to_jsvalue)?
    } else {
        secret_key // If no passphrase, key is not encrypted (less secure)
    };
    
    // Armor the keys
    let mut public_key_armored_writer = Vec::new();
    final_secret_key.public_key().armor(ArmorOptions::new(ArmorType::PublicKey, Default::default()))
        .write_to(&mut public_key_armored_writer)
        .map_err(pgp_error_to_jsvalue)?;
    let public_key_armored = String::from_utf8(public_key_armored_writer)
        .map_err(|e| JsValue::from_str(&format!("UTF-8 conversion error for public key: {}", e)))?;

    let mut private_key_armored_writer = Vec::new();
    final_secret_key.armor(ArmorOptions::new(ArmorType::PrivateKey, Default::default()))
        .write_to(&mut private_key_armored_writer)
        .map_err(pgp_error_to_jsvalue)?;
    let private_key_armored = String::from_utf8(private_key_armored_writer)
        .map_err(|e| JsValue::from_str(&format!("UTF-8 conversion error for private key: {}", e)))?;

    let public_key = final_secret_key.public_key();
    let key_id = public_key.key_id().to_string();
    let fingerprint = public_key.fingerprint().to_string();
    // TODO: Determine how to get the primary algorithm string correctly.
    // For now, hardcoding based on selection.
    let algorithm_string = format!("{:?}/{:?}", 
        public_key.algorithm(), 
        public_key.subkeys().get(0).map_or(PublicKeyAlgorithm::None, |sk| sk.algorithm())
    );


    let result = KeyPairResult {
        key_id,
        fingerprint,
        user_id: params.user_id.clone(), // Use the original user_id string
        algorithm: algorithm_string,
        public_key_armored,
        private_key_armored,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    log("✅ Key pair generated successfully using pgp crate.");
    
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn encrypt_message(params_json: &str) -> Result<String, JsValue> {
    log("🔒 Encrypting message with pgp crate...");
    
    let params: EncryptParams = serde_json::from_str(params_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid parameters: {}", e)))?;

    let mut recipient_keys = Vec::new();
    for armored_key_str in params.recipient_public_keys {
        let (key, _headers) = SignedPublicKey::from_string(&armored_key_str)
            .map_err(pgp_error_to_jsvalue)?;
        // Ensure the key is suitable for encryption
        if !key.is_encryption_key() {
            return Err(JsValue::from_str(&format!("Key ID {} is not an encryption key.", key.key_id())));
        }
        recipient_keys.push(key);
    }

    if recipient_keys.is_empty() {
        return Err(JsValue::from_str("No valid recipient keys provided for encryption."));
    }

    // Create a message from the plaintext.
    // The pgp crate's Message::new_literal_bytes is suitable for arbitrary byte data.
    // For text, ensure it's UTF-8.
    let message_body = params.plaintext.as_bytes();
    let pgp_message = Message::new_literal_bytes("message.txt", message_body);
    
    // TODO: Add signing capabilities if signing_secret_key_armored is provided.
    // This would involve parsing the signing key, unlocking it, and creating a SignedMessage.
    // For now, just encrypting.

    let encrypted_message = pgp_message.encrypt_to_keys(
            &mut rand::thread_rng(), // Random number generator
            PublicKeyAlgorithm::Kyber1024, // This might be derived from key properties or set explicitly
            SymmetricKeyAlgorithm::AES256, // Preferred symmetric algorithm
            &recipient_keys,
        )
        .map_err(pgp_error_to_jsvalue)?;

    // Armor the encrypted message
    let mut armored_ciphertext_writer = Vec::new();
    encrypted_message.armor(ArmorOptions::new(ArmorType::Message, Default::default()))
        .write_to(&mut armored_ciphertext_writer)
        .map_err(pgp_error_to_jsvalue)?;
    let armored_ciphertext = String::from_utf8(armored_ciphertext_writer)
        .map_err(|e| JsValue::from_str(&format!("UTF-8 conversion error for ciphertext: {}", e)))?;
    
    log("✅ Message encrypted successfully using pgp crate.");
    Ok(armored_ciphertext)
}

#[wasm_bindgen]
pub fn decrypt_message(params_json: &str) -> Result<String, JsValue> {
    log("🔓 Decrypting message with pgp crate...");
    
    let params: DecryptParams = serde_json::from_str(params_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid parameters: {}", e)))?;

    // Dearmor the ciphertext
    let (message, _headers) = Message::from_string(&params.ciphertext)
        .map_err(pgp_error_to_jsvalue)?;

    // Parse the armored private key
    let (mut secret_key, _headers) = SignedSecretKey::from_string(&params.armored_private_key)
        .map_err(pgp_error_to_jsvalue)?;

    // Unlock the private key if it's encrypted and a passphrase is provided
    if secret_key.is_encrypted() {
        if params.passphrase.is_empty() {
            return Err(JsValue::from_str("Private key is encrypted, but no passphrase was provided."));
        }
        let pw = Password::new(&params.passphrase);
        secret_key.unlock(&pw, Default::default()).map_err(pgp_error_to_jsvalue)?;
    } else if !secret_key.is_encrypted() && !params.passphrase.is_empty() {
        // Optional: could warn if passphrase provided for unencrypted key, but generally not an error.
        log("Provided passphrase for an unencrypted key. Proceeding with decryption.");
    }


    // Decrypt the message
    // The decrypt_message function in the pgp crate typically requires a list of secret keys
    // and will find the one that can decrypt the message.
    let (plaintext_bytes, _session_key_algo, _sym_key_algo) = message
        .decrypt(&[&secret_key], &[]) // Second arg is for session keys, not typically needed here
        .map_err(pgp_error_to_jsvalue)?;
    
    let plaintext = String::from_utf8(plaintext_bytes)
        .map_err(|e| JsValue::from_str(&format!("UTF-8 conversion error for plaintext: {}", e)))?;
    
    log("✅ Message decrypted successfully using pgp crate.");
    Ok(plaintext)
}

#[wasm_bindgen]
pub fn sign_message(params_json: &str) -> Result<String, JsValue> {
    log("✍️ Signing message with pgp crate...");
    
    let params: SignParams = serde_json::from_str(params_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid parameters: {}", e)))?;

    // Parse the armored private key
    let (mut secret_key, _headers) = SignedSecretKey::from_string(&params.armored_private_key)
        .map_err(pgp_error_to_jsvalue)?;

    // Unlock the private key
    if secret_key.is_encrypted() {
        if params.passphrase.is_empty() {
            return Err(JsValue::from_str("Private key is encrypted, but no passphrase was provided."));
        }
        let pw = Password::new(&params.passphrase);
        secret_key.unlock(&pw, Default::default()).map_err(pgp_error_to_jsvalue)?;
    } else if !secret_key.is_encrypted() && !params.passphrase.is_empty() {
        log("Provided passphrase for an unencrypted key. Proceeding with signing.");
    }
    
    // The message to sign
    let message_bytes = params.message.as_bytes();

    // Create a cleartext signed message
    // Note: The pgp crate typically signs data and then can represent it in various forms (cleartext, detached).
    // For cleartext signed messages, the library usually handles the specific formatting.
    // We'll create a signature, then construct the armored output.

    let (signature_packet, _hash_algo) = secret_key.sign_message(
        &mut rand::thread_rng(),
        message_bytes,
        HashAlgorithm::SHA512, // Should align with key preferences or be chosen carefully
        SystemTime::now()
    ).map_err(pgp_error_to_jsvalue)?;


    let mut armored_signed_message_writer = Vec::new();
    // For a cleartext signed message, we need to write the message, then the signature.
    // The `pgp` crate's `armor` function on a signature packet itself creates a detached signature.
    // To create a PGP CLEARMESSAGE (like in GnuPG `gpg --clearsign`), we manually construct it.
    // This involves specific BEGIN/END PGP SIGNED MESSAGE blocks and armoring the signature.
    // However, the `pgp` crate's `Message::sign_cleartext` is what we need if available,
    // or we construct it manually. Let's assume for now the request is for a simple armored signature
    // if `Message::sign_cleartext` isn't straightforward or if a combined message is complex.
    // Re-evaluating: `SignedMessage::new_cleartext` seems to be the way.

    let signed_message = Message::new_literal_bytes("message.txt", message_bytes)
        .sign_cleartext(&secret_key, SignatureType::CanonicalizedText, HashAlgorithm::SHA512, SystemTime::now())
        .map_err(pgp_error_to_jsvalue)?;


    let mut writer = Vec::new();
    // For cleartext signed messages, the armoring needs to be specific.
    // The `pgp` crate doesn't seem to have a direct high-level "armor cleartext signed message".
    // Typically, you'd get the signature and then manually construct the ASCII output or
    // use a lower-level API if available.
    // Given the constraints, returning a detached signature might be more straightforward if cleartext signing is complex.
    // Let's try to produce the combined signed message format.
    // The `SignedMessage` type has an `armor` method.

    signed_message.armor(ArmorOptions::new(ArmorType::SignedMessage, Default::default()))
        .write_to(&mut writer)
        .map_err(pgp_error_to_jsvalue)?;

    let result_string = String::from_utf8(writer)
        .map_err(|e| JsValue::from_str(&format!("UTF-8 conversion error: {}", e)))?;

    log("✅ Message signed (cleartext) successfully using pgp crate.");
    Ok(result_string)
}

#[wasm_bindgen]
pub fn create_detached_signature(params_json: &str) -> Result<String, JsValue> {
    log("✍️ Creating detached signature with pgp crate...");
    
    let params: SignParams = serde_json::from_str(params_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid parameters: {}", e)))?;

    let (mut secret_key, _headers) = SignedSecretKey::from_string(&params.armored_private_key)
        .map_err(pgp_error_to_jsvalue)?;

    if secret_key.is_encrypted() {
        if params.passphrase.is_empty() {
            return Err(JsValue::from_str("Private key is encrypted, but no passphrase was provided."));
        }
        let pw = Password::new(&params.passphrase);
        secret_key.unlock(&pw, Default::default()).map_err(pgp_error_to_jsvalue)?;
    } else if !secret_key.is_encrypted() && !params.passphrase.is_empty() {
        log("Provided passphrase for an unencrypted key. Proceeding with signing.");
    }

    let message_bytes = params.message.as_bytes();
    
    // Create a detached signature
    let (signature_packet, _hash_algo) = secret_key.sign_message(
        &mut rand::thread_rng(),
        message_bytes,
        HashAlgorithm::SHA512, // Or derive from key preferences
        SystemTime::now()
    ).map_err(pgp_error_to_jsvalue)?;

    let mut armored_signature_writer = Vec::new();
    signature_packet.armor(ArmorOptions::new(ArmorType::Signature, Default::default()))
        .write_to(&mut armored_signature_writer)
        .map_err(pgp_error_to_jsvalue)?;
    let armored_signature = String::from_utf8(armored_signature_writer)
        .map_err(|e| JsValue::from_str(&format!("UTF-8 conversion error for signature: {}", e)))?;
    
    log("✅ Detached signature created successfully using pgp crate.");
    Ok(armored_signature)
}

#[wasm_bindgen]
pub fn verify_message(params_json: &str) -> Result<String, JsValue> {
    log("🔍 Verifying signature with pgp crate...");
    
    let params: VerifyParams = serde_json::from_str(params_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid parameters: {}", e)))?;

    // Parse the armored public key
    let (public_key, _headers) = SignedPublicKey::from_string(&params.armored_public_key)
        .map_err(pgp_error_to_jsvalue)?;

    // The message bytes
    let message_bytes = params.message.as_bytes();

    // Attempt to parse the signature. It could be a detached signature or part of a clearsigned message.
    // The `pgp` crate handles both cases through `Message::from_string` for clearsigned,
    // or `Signature::from_string` for detached.
    // For verification, we typically need to provide the data that was signed.

    // Try parsing as a clearsigned message first
    match Message::from_string(&params.signature) {
        Ok((signed_message, _headers)) => {
            // It's a PGP message, possibly clearsigned or just a signature block
            match signed_message.verify_cleartext(&public_key, message_bytes) {
                Ok(_) => {
                    let verify_result = VerifyResult {
                        is_valid: true,
                        message: "✅ Signature verified successfully (clearsigned/embedded).".to_string(),
                    };
                    log(&verify_result.message);
                    return serde_json::to_string(&verify_result)
                        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)));
                }
                Err(e) => {
                     // If verify_cleartext fails, it might be because it's not a clearsigned message
                     // but a detached signature, or verification genuinely failed.
                     log(&format!("Clearsigned verification attempt failed: {}. Trying as detached.", e));
                     // Fall through to detached signature verification
                }
            }
        }
        Err(_) => {
            // Not a valid PGP message armor, likely a detached signature or invalid
            log("Not a full PGP message armor, attempting to parse as detached signature.");
        }
    }
    
    // Try parsing as a detached signature
    let (detached_signature, _headers) = pgp::composed::Signature::from_string(&params.signature)
        .map_err(|e| pgp_error_to_jsvalue(e).into_message(&format!("Failed to parse signature: {}", e)))?;
        // If from_string fails, it means the signature block is malformed.

    match public_key.verify_message(message_bytes, &detached_signature) {
        Ok(_) => {
            let verify_result = VerifyResult {
                is_valid: true,
                message: "✅ Signature verified successfully (detached).".to_string(),
            };
            log(&verify_result.message);
            serde_json::to_string(&verify_result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        }
        Err(e) => {
            let verify_result = VerifyResult {
                is_valid: false,
                message: format!("❌ Signature verification failed: {}", e),
            };
            log(&verify_result.message);
            serde_json::to_string(&verify_result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        }
    }
}