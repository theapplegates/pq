use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use rpgp::composed::{Deserializable, KeyType, SecretKeyParamsBuilder, SubkeyParamsBuilder};
use rpgp::crypto::hash::HashAlgorithm;
use rpgp::crypto::public_key::PublicKeyAlgorithm;
use rpgp::crypto::sym::SymmetricKeyAlgorithm;
use rpgp::packet::{SignatureType, UserAttribute};
use rpgp::{Deserializable as _, Message, SignedPublicKey, SignedSecretKey};
use std::io::Cursor;

// Set up panic hook for better error messages
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
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
pub struct PublicKeyResult {
    pub key_id: String,
    pub fingerprint: String,
    pub user_id: String,
    pub algorithm: String,
    pub public_key_armored: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct GenerateKeyParams {
    pub user_id: String,
    pub passphrase: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EncryptParams {
    pub recipient_key_ids: Vec<String>,
    pub plaintext: String,
}

#[derive(Serialize, Deserialize)]
pub struct DecryptParams {
    pub private_key_id: String,
    pub passphrase: Option<String>,
    pub ciphertext: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignParams {
    pub private_key_id: String,
    pub passphrase: Option<String>,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyParams {
    pub signer_key_id: String,
    pub message: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyResult {
    pub is_valid: bool,
    pub message: String,
}

// Global storage for keys (in a real implementation, you'd want proper storage)
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref KEY_STORAGE: Mutex<HashMap<String, SignedSecretKey>> = Mutex::new(HashMap::new());
    static ref PUBLIC_KEY_STORAGE: Mutex<HashMap<String, SignedPublicKey>> = Mutex::new(HashMap::new());
}

#[wasm_bindgen]
pub fn generate_key_pair(params_js: JsValue) -> Promise {
    future_to_promise(async move {
        let params: GenerateKeyParams = serde_wasm_bindgen::from_value(params_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse params: {}", e)))?;

        // Create primary key for signing with Dilithium5
        let primary_key_params = SecretKeyParamsBuilder::default()
            .key_type(KeyType::Dilithium5) // Post-quantum signature algorithm
            .can_certify(true)
            .can_sign(true)
            .primary_user_id(params.user_id.clone())
            .passphrase(params.passphrase.clone())
            .preferred_symmetric_algorithms(smallvec::smallvec![
                SymmetricKeyAlgorithm::AES256,
                SymmetricKeyAlgorithm::AES192,
                SymmetricKeyAlgorithm::AES128,
            ])
            .preferred_hash_algorithms(smallvec::smallvec![
                HashAlgorithm::SHA2_256,
                HashAlgorithm::SHA2_384,
                HashAlgorithm::SHA2_512,
            ])
            .build()
            .map_err(|e| JsValue::from_str(&format!("Failed to build primary key params: {}", e)))?;

        // Create encryption subkey with Kyber1024
        let subkey_params = SubkeyParamsBuilder::default()
            .key_type(KeyType::Kyber1024) // Post-quantum encryption algorithm
            .can_encrypt(true)
            .passphrase(params.passphrase.clone())
            .build()
            .map_err(|e| JsValue::from_str(&format!("Failed to build subkey params: {}", e)))?;

        // Generate the key pair
        let secret_key = primary_key_params
            .generate()
            .map_err(|e| JsValue::from_str(&format!("Failed to generate primary key: {}", e)))?;

        let secret_key_with_subkey = secret_key
            .add_subkey(subkey_params)
            .map_err(|e| JsValue::from_str(&format!("Failed to add subkey: {}", e)))?;

        // Get key information
        let key_id = hex::encode(secret_key_with_subkey.primary_key.key_id());
        let fingerprint = hex::encode(secret_key_with_subkey.primary_key.fingerprint());
        let public_key = secret_key_with_subkey.public_key();

        // Convert to armored format
        let private_key_armored = secret_key_with_subkey
            .to_armored_string(None)
            .map_err(|e| JsValue::from_str(&format!("Failed to armor private key: {}", e)))?;

        let public_key_armored = public_key
            .to_armored_string(None)
            .map_err(|e| JsValue::from_str(&format!("Failed to armor public key: {}", e)))?;

        // Store keys
        {
            let mut storage = KEY_STORAGE.lock().unwrap();
            storage.insert(key_id.clone(), secret_key_with_subkey);
        }
        {
            let mut pub_storage = PUBLIC_KEY_STORAGE.lock().unwrap();
            pub_storage.insert(key_id.clone(), public_key);
        }

        let result = KeyPairResult {
            key_id,
            fingerprint,
            user_id: params.user_id,
            algorithm: "Dilithium5 (signing) + Kyber1024 (encryption)".to_string(),
            public_key_armored,
            private_key_armored,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(serde_wasm_bindgen::to_value(&result)?)
    })
}

#[wasm_bindgen]
pub fn get_public_key(key_id: String) -> Promise {
    future_to_promise(async move {
        let storage = PUBLIC_KEY_STORAGE.lock().unwrap();
        
        if let Some(public_key) = storage.get(&key_id) {
            let public_key_armored = public_key
                .to_armored_string(None)
                .map_err(|e| JsValue::from_str(&format!("Failed to armor public key: {}", e)))?;

            let user_id = public_key
                .users()
                .next()
                .map(|u| u.id().clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let fingerprint = hex::encode(public_key.primary_key.fingerprint());

            let result = PublicKeyResult {
                key_id: key_id.clone(),
                fingerprint,
                user_id,
                algorithm: "Dilithium5 (signing) + Kyber1024 (encryption)".to_string(),
                public_key_armored,
                created_at: chrono::Utc::now().to_rfc3339(), // Would need to store actual creation time
            };

            Ok(serde_wasm_bindgen::to_value(&result)?)
        } else {
            Err(JsValue::from_str("Key not found"))
        }
    })
}

#[wasm_bindgen]
pub fn get_all_public_keys() -> Promise {
    future_to_promise(async move {
        let storage = PUBLIC_KEY_STORAGE.lock().unwrap();
        let mut results = Vec::new();

        for (key_id, public_key) in storage.iter() {
            let public_key_armored = public_key
                .to_armored_string(None)
                .map_err(|e| JsValue::from_str(&format!("Failed to armor public key: {}", e)))?;

            let user_id = public_key
                .users()
                .next()
                .map(|u| u.id().clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let fingerprint = hex::encode(public_key.primary_key.fingerprint());

            results.push(PublicKeyResult {
                key_id: key_id.clone(),
                fingerprint,
                user_id,
                algorithm: "Dilithium5 (signing) + Kyber1024 (encryption)".to_string(),
                public_key_armored,
                created_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        Ok(serde_wasm_bindgen::to_value(&results)?)
    })
}

#[wasm_bindgen]
pub fn encrypt_message(params_js: JsValue) -> Promise {
    future_to_promise(async move {
        let params: EncryptParams = serde_wasm_bindgen::from_value(params_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse params: {}", e)))?;

        let pub_storage = PUBLIC_KEY_STORAGE.lock().unwrap();
        let mut recipient_keys = Vec::new();

        // Collect recipient public keys
        for key_id in &params.recipient_key_ids {
            if let Some(pub_key) = pub_storage.get(key_id) {
                recipient_keys.push(pub_key);
            } else {
                return Err(JsValue::from_str(&format!("Recipient key not found: {}", key_id)));
            }
        }

        if recipient_keys.is_empty() {
            return Err(JsValue::from_str("No valid recipient keys found"));
        }

        // Create message
        let msg = Message::new_literal("", &params.plaintext);

        // Encrypt the message
        let encrypted_msg = msg
            .encrypt_to_keys(&mut rand::thread_rng(), SymmetricKeyAlgorithm::AES256, &recipient_keys)
            .map_err(|e| JsValue::from_str(&format!("Failed to encrypt message: {}", e)))?;

        // Convert to armored string
        let ciphertext = encrypted_msg
            .to_armored_string(None)
            .map_err(|e| JsValue::from_str(&format!("Failed to armor encrypted message: {}", e)))?;

        Ok(JsValue::from_str(&ciphertext))
    })
}

#[wasm_bindgen]
pub fn decrypt_message(params_js: JsValue) -> Promise {
    future_to_promise(async move {
        let params: DecryptParams = serde_wasm_bindgen::from_value(params_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse params: {}", e)))?;

        let storage = KEY_STORAGE.lock().unwrap();
        
        let secret_key = storage
            .get(&params.private_key_id)
            .ok_or_else(|| JsValue::from_str("Private key not found"))?;

        // Parse the encrypted message
        let (encrypted_msg, _) = Message::from_armor_single(Cursor::new(params.ciphertext.as_bytes()))
            .map_err(|e| JsValue::from_str(&format!("Failed to parse encrypted message: {}", e)))?;

        // Decrypt the message
        let (decrypted_msg, _) = encrypted_msg
            .decrypt(|| params.passphrase.clone().unwrap_or_default(), &[secret_key])
            .map_err(|e| JsValue::from_str(&format!("Failed to decrypt message: {}", e)))?;

        // Extract plaintext
        let plaintext = String::from_utf8(decrypted_msg.get_content().unwrap_or_default())
            .map_err(|e| JsValue::from_str(&format!("Failed to convert decrypted content to string: {}", e)))?;

        Ok(JsValue::from_str(&plaintext))
    })
}

#[wasm_bindgen]
pub fn sign_message(params_js: JsValue) -> Promise {
    future_to_promise(async move {
        let params: SignParams = serde_wasm_bindgen::from_value(params_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse params: {}", e)))?;

        let storage = KEY_STORAGE.lock().unwrap();
        
        let secret_key = storage
            .get(&params.private_key_id)
            .ok_or_else(|| JsValue::from_str("Private key not found"))?;

        // Create message
        let msg = Message::new_literal("", &params.message);

        // Sign the message (clear-sign)
        let signed_msg = msg
            .sign(secret_key, || params.passphrase.clone().unwrap_or_default(), HashAlgorithm::SHA2_256)
            .map_err(|e| JsValue::from_str(&format!("Failed to sign message: {}", e)))?;

        // Convert to armored string
        let signed_armored = signed_msg
            .to_armored_string(None)
            .map_err(|e| JsValue::from_str(&format!("Failed to armor signed message: {}", e)))?;

        Ok(JsValue::from_str(&signed_armored))
    })
}

#[wasm_bindgen]
pub fn create_detached_signature(params_js: JsValue) -> Promise {
    future_to_promise(async move {
        let params: SignParams = serde_wasm_bindgen::from_value(params_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse params: {}", e)))?;

        let storage = KEY_STORAGE.lock().unwrap();
        
        let secret_key = storage
            .get(&params.private_key_id)
            .ok_or_else(|| JsValue::from_str("Private key not found"))?;

        // Create detached signature
        let signature = secret_key
            .create_signature(
                || params.passphrase.clone().unwrap_or_default(),
                HashAlgorithm::SHA2_256,
                params.message.as_bytes(),
            )
            .map_err(|e| JsValue::from_str(&format!("Failed to create signature: {}", e)))?;

        // Convert to armored string
        let signature_armored = signature
            .to_armored_string(None)
            .map_err(|e| JsValue::from_str(&format!("Failed to armor signature: {}", e)))?;

        Ok(JsValue::from_str(&signature_armored))
    })
}

#[wasm_bindgen]
pub fn verify_message(params_js: JsValue) -> Promise {
    future_to_promise(async move {
        let params: VerifyParams = serde_wasm_bindgen::from_value(params_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse params: {}", e)))?;

        let pub_storage = PUBLIC_KEY_STORAGE.lock().unwrap();
        
        let public_key = pub_storage
            .get(&params.signer_key_id)
            .ok_or_else(|| JsValue::from_str("Signer's public key not found"))?;

        // Parse the signature
        let (signature, _) = rpgp::packet::Signature::from_armor_single(Cursor::new(params.signature.as_bytes()))
            .map_err(|e| JsValue::from_str(&format!("Failed to parse signature: {}", e)))?;

        // Verify the signature
        let is_valid = signature
            .verify(&public_key.primary_key, params.message.as_bytes())
            .is_ok();

        let result = VerifyResult {
            is_valid,
            message: if is_valid {
                format!("Signature verification SUCCESSFUL with key {}", params.signer_key_id)
            } else {
                format!("Signature verification FAILED with key {}", params.signer_key_id)
            },
        };

        Ok(serde_wasm_bindgen::to_value(&result)?)
    })
}