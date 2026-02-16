// File: src/interpreter/native_functions/crypto.rs
//
// Cryptography native functions

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use md5::Md5;
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::{Oaep, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};

use crate::interpreter::{DictMap, Value};
use std::sync::Arc;

fn error_object(message: String) -> Value {
    Value::ErrorObject { message, stack: Vec::new(), line: None, cause: None }
}

pub fn handle(name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        "sha256" => {
            if let Some(Value::Str(data)) = arg_values.first() {
                let mut hasher = Sha256::new();
                hasher.update(data.as_bytes());
                let result = hasher.finalize();
                Value::Str(Arc::new(format!("{:x}", result)))
            } else {
                Value::Error("sha256 requires a string argument".to_string())
            }
        }

        "md5" => {
            if let Some(Value::Str(data)) = arg_values.first() {
                let mut hasher = Md5::new();
                hasher.update(data.as_bytes());
                let result = hasher.finalize();
                Value::Str(Arc::new(format!("{:x}", result)))
            } else {
                Value::Error("md5 requires a string argument".to_string())
            }
        }

        "md5_file" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::read(path.as_ref()) {
                    Ok(contents) => {
                        let mut hasher = Md5::new();
                        hasher.update(&contents);
                        let result = hasher.finalize();
                        Value::Str(Arc::new(format!("{:x}", result)))
                    }
                    Err(e) => {
                        error_object(format!("Failed to read file '{}': {}", path.as_ref(), e))
                    }
                }
            } else {
                Value::Error("md5_file requires a string path argument".to_string())
            }
        }

        "hash_password" => {
            if let Some(Value::Str(password)) = arg_values.first() {
                match bcrypt::hash(password.as_ref(), bcrypt::DEFAULT_COST) {
                    Ok(hashed) => Value::Str(Arc::new(hashed)),
                    Err(e) => error_object(format!("Failed to hash password: {}", e)),
                }
            } else {
                Value::Error("hash_password requires a string password argument".to_string())
            }
        }

        "verify_password" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Str(password)), Some(Value::Str(hash))) => {
                match bcrypt::verify(password.as_ref(), hash.as_ref()) {
                    Ok(is_valid) => Value::Bool(is_valid),
                    Err(e) => error_object(format!("Failed to verify password: {}", e)),
                }
            }
            _ => Value::Error(
                "verify_password requires (string_password, string_hash) arguments".to_string(),
            ),
        },

        "aes_encrypt" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Str(plaintext)), Some(Value::Str(key))) => {
                let mut hasher = Sha256::new();
                hasher.update(key.as_bytes());
                let key_bytes = hasher.finalize();

                let nonce_bytes: [u8; 12] = rand::random();
                let nonce = Nonce::from_slice(&nonce_bytes);

                match Aes256Gcm::new_from_slice(&key_bytes) {
                    Ok(cipher) => match cipher.encrypt(nonce, plaintext.as_bytes()) {
                        Ok(ciphertext) => {
                            let mut result = nonce_bytes.to_vec();
                            result.extend_from_slice(&ciphertext);
                            Value::Str(Arc::new(base64::engine::general_purpose::STANDARD.encode(result)))
                        }
                        Err(e) => error_object(format!("AES encryption failed: {}", e)),
                    },
                    Err(e) => error_object(format!("Failed to create AES cipher: {}", e)),
                }
            }
            _ => {
                Value::Error("aes_encrypt requires (plaintext_string, key_string) arguments".to_string())
            }
        },

        "aes_decrypt" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Str(ciphertext_b64)), Some(Value::Str(key))) => {
                let mut hasher = Sha256::new();
                hasher.update(key.as_bytes());
                let key_bytes = hasher.finalize();

                match base64::engine::general_purpose::STANDARD.decode(ciphertext_b64.as_ref()) {
                    Ok(data) => {
                        if data.len() < 12 {
                            return Some(Value::Error("Invalid ciphertext: too short".to_string()));
                        }

                        let nonce = Nonce::from_slice(&data[..12]);
                        let ciphertext = &data[12..];

                        match Aes256Gcm::new_from_slice(&key_bytes) {
                            Ok(cipher) => match cipher.decrypt(nonce, ciphertext) {
                                Ok(plaintext) => match String::from_utf8(plaintext) {
                                    Ok(s) => Value::Str(Arc::new(s)),
                                    Err(e) => {
                                        error_object(format!("Decrypted data is not valid UTF-8: {}", e))
                                    }
                                },
                                Err(e) => error_object(format!("AES decryption failed: {}", e)),
                            },
                            Err(e) => error_object(format!("Failed to create AES cipher: {}", e)),
                        }
                    }
                    Err(e) => error_object(format!("Invalid base64 ciphertext: {}", e)),
                }
            }
            _ => Value::Error(
                "aes_decrypt requires (ciphertext_string, key_string) arguments".to_string(),
            ),
        },

        "aes_encrypt_bytes" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Str(data)), Some(Value::Str(key))) => {
                let mut hasher = Sha256::new();
                hasher.update(key.as_bytes());
                let key_bytes = hasher.finalize();

                let nonce_bytes: [u8; 12] = rand::random();
                let nonce = Nonce::from_slice(&nonce_bytes);

                match Aes256Gcm::new_from_slice(&key_bytes) {
                    Ok(cipher) => match cipher.encrypt(nonce, data.as_bytes()) {
                        Ok(ciphertext) => {
                            let mut result = nonce_bytes.to_vec();
                            result.extend_from_slice(&ciphertext);
                            Value::Str(Arc::new(base64::engine::general_purpose::STANDARD.encode(result)))
                        }
                        Err(e) => error_object(format!("AES encryption failed: {}", e)),
                    },
                    Err(e) => error_object(format!("Failed to create AES cipher: {}", e)),
                }
            }
            _ => {
                Value::Error("aes_encrypt_bytes requires (data_string, key_string) arguments".to_string())
            }
        },

        "aes_decrypt_bytes" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Str(ciphertext_b64)), Some(Value::Str(key))) => {
                let mut hasher = Sha256::new();
                hasher.update(key.as_bytes());
                let key_bytes = hasher.finalize();

                match base64::engine::general_purpose::STANDARD.decode(ciphertext_b64.as_ref()) {
                    Ok(data) => {
                        if data.len() < 12 {
                            return Some(Value::Error("Invalid ciphertext: too short".to_string()));
                        }

                        let nonce = Nonce::from_slice(&data[..12]);
                        let ciphertext = &data[12..];

                        match Aes256Gcm::new_from_slice(&key_bytes) {
                            Ok(cipher) => match cipher.decrypt(nonce, ciphertext) {
                                Ok(plaintext) => match String::from_utf8(plaintext.clone()) {
                                    Ok(s) => Value::Str(Arc::new(s)),
                                    Err(_) => Value::Str(Arc::new(
                                        base64::engine::general_purpose::STANDARD
                                            .encode(&plaintext),
                                    )),
                                },
                                Err(e) => error_object(format!("AES decryption failed: {}", e)),
                            },
                            Err(e) => error_object(format!("Failed to create AES cipher: {}", e)),
                        }
                    }
                    Err(e) => error_object(format!("Invalid base64 ciphertext: {}", e)),
                }
            }
            _ => Value::Error(
                "aes_decrypt_bytes requires (ciphertext_string, key_string) arguments"
                    .to_string(),
            ),
        },

        "rsa_generate_keypair" => {
            if let Some(Value::Int(bits)) = arg_values.first() {
                let bits_usize = *bits as usize;
                if bits_usize != 2048 && bits_usize != 4096 {
                    return Some(Value::Error("RSA key size must be 2048 or 4096 bits".to_string()));
                }

                let mut rng = rand::thread_rng();
                match RsaPrivateKey::new(&mut rng, bits_usize) {
                    Ok(private_key) => {
                        let public_key = RsaPublicKey::from(&private_key);

                        let private_pem = match private_key.to_pkcs8_pem(LineEnding::LF) {
                            Ok(pem) => pem.to_string(),
                            Err(e) => {
                                return Some(error_object(format!(
                                    "Failed to encode private key: {}",
                                    e
                                )))
                            }
                        };

                        let public_pem = match public_key.to_public_key_pem(LineEnding::LF) {
                            Ok(pem) => pem,
                            Err(e) => {
                                return Some(error_object(format!(
                                    "Failed to encode public key: {}",
                                    e
                                )))
                            }
                        };

                        let mut keypair = DictMap::default();
                        keypair.insert(Arc::<str>::from("private"), Value::Str(Arc::new(private_pem)));
                        keypair.insert(Arc::<str>::from("public"), Value::Str(Arc::new(public_pem)));
                        Value::Dict(Arc::new(keypair))
                    }
                    Err(e) => error_object(format!("Failed to generate RSA keypair: {}", e)),
                }
            } else {
                Value::Error("rsa_generate_keypair requires an integer (2048 or 4096)".to_string())
            }
        }

        "rsa_encrypt" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Str(plaintext)), Some(Value::Str(public_key_pem))) => {
                match RsaPublicKey::from_public_key_pem(public_key_pem.as_ref()) {
                    Ok(public_key) => {
                        let mut rng = rand::thread_rng();
                        let padding = Oaep::new::<Sha256>();

                        match public_key.encrypt(&mut rng, padding, plaintext.as_bytes()) {
                            Ok(ciphertext) => Value::Str(Arc::new(
                                base64::engine::general_purpose::STANDARD.encode(ciphertext),
                            )),
                            Err(e) => error_object(format!("RSA encryption failed: {}", e)),
                        }
                    }
                    Err(e) => error_object(format!("Invalid RSA public key: {}", e)),
                }
            }
            _ => Value::Error(
                "rsa_encrypt requires (plaintext_string, public_key_pem) arguments".to_string(),
            ),
        },

        "rsa_decrypt" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Str(ciphertext_b64)), Some(Value::Str(private_key_pem))) => {
                match RsaPrivateKey::from_pkcs8_pem(private_key_pem.as_ref()) {
                    Ok(private_key) => {
                        match base64::engine::general_purpose::STANDARD
                            .decode(ciphertext_b64.as_ref())
                        {
                            Ok(ciphertext) => {
                                let padding = Oaep::new::<Sha256>();
                                match private_key.decrypt(padding, &ciphertext) {
                                    Ok(plaintext) => match String::from_utf8(plaintext) {
                                        Ok(s) => Value::Str(Arc::new(s)),
                                        Err(e) => {
                                            error_object(format!("Decrypted data is not valid UTF-8: {}", e))
                                        }
                                    },
                                    Err(e) => error_object(format!("RSA decryption failed: {}", e)),
                                }
                            }
                            Err(e) => error_object(format!("Invalid base64 ciphertext: {}", e)),
                        }
                    }
                    Err(e) => error_object(format!("Invalid RSA private key: {}", e)),
                }
            }
            _ => Value::Error(
                "rsa_decrypt requires (ciphertext_string, private_key_pem) arguments".to_string(),
            ),
        },

        "rsa_sign" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Str(message)), Some(Value::Str(private_key_pem))) => {
                match RsaPrivateKey::from_pkcs8_pem(private_key_pem.as_ref()) {
                    Ok(private_key) => {
                        use rsa::pkcs1v15::SigningKey;
                        use rsa::signature::{SignatureEncoding, Signer};

                        let signing_key = SigningKey::<Sha256>::new(private_key);
                        let signature = signing_key.sign(message.as_bytes());
                        Value::Str(Arc::new(
                            base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()),
                        ))
                    }
                    Err(e) => error_object(format!("Invalid RSA private key: {}", e)),
                }
            }
            _ => Value::Error(
                "rsa_sign requires (message_string, private_key_pem) arguments".to_string(),
            ),
        },

        "rsa_verify" => match (arg_values.first(), arg_values.get(1), arg_values.get(2)) {
            (Some(Value::Str(message)), Some(Value::Str(signature_b64)), Some(Value::Str(public_key_pem))) => {
                match RsaPublicKey::from_public_key_pem(public_key_pem.as_ref()) {
                    Ok(public_key) => match base64::engine::general_purpose::STANDARD
                        .decode(signature_b64.as_ref())
                    {
                        Ok(signature_bytes) => {
                            use rsa::pkcs1v15::{Signature, VerifyingKey};
                            use rsa::signature::Verifier;

                            let verifying_key = VerifyingKey::<Sha256>::new(public_key);

                            match Signature::try_from(signature_bytes.as_slice()) {
                                Ok(signature) => match verifying_key.verify(message.as_bytes(), &signature) {
                                    Ok(_) => Value::Bool(true),
                                    Err(_) => Value::Bool(false),
                                },
                                Err(e) => error_object(format!("Invalid signature format: {}", e)),
                            }
                        }
                        Err(e) => error_object(format!("Invalid base64 signature: {}", e)),
                    },
                    Err(e) => error_object(format!("Invalid RSA public key: {}", e)),
                }
            }
            _ => Value::Error(
                "rsa_verify requires (message, signature, public_key_pem) arguments".to_string(),
            ),
        },

        _ => return None,
    };

    Some(result)
}
