use std::io::{Read, Write};

use anyhow::Result;
use openssl::{
    hash::{hash, MessageDigest},
    symm::{decrypt, encrypt, Cipher},
};
use pqcrypto::traits::{
    kem::{Ciphertext, PublicKey, SharedSecret},
    sign::DetachedSignature,
};
use serde_json::Value;

/// Get the cipher to use
fn cipher() -> Cipher {
    Cipher::aes_256_cbc()
}

pub struct Encrypted<T> {
    stream: T,
    shared_secret: crate::key_exchange::SharedSecret,
    cipher: Cipher,
    iv: Vec<u8>,
}

impl<T> Encrypted<T>
where
    T: Read + Write,
{
    pub fn request(mut stream: T) -> Result<Self> {
        let (pk, sk) = crate::key_exchange::keypair();
        stream.write_all(pk.as_bytes())?;
        let mut ct_bytes = [0; crate::key_exchange::ciphertext_bytes()];
        stream.read_exact(&mut ct_bytes)?;
        let ct = crate::key_exchange::Ciphertext::from_bytes(&ct_bytes)?;
        let ss = crate::key_exchange::decapsulate(&ct, &sk);
        let cipher = cipher();
        let digest = MessageDigest::shake_128();
        let iv = hash(digest, ss.as_bytes())?.to_vec(); // Create iv from shared secret
        Ok(Self {
            stream,
            shared_secret: ss,
            cipher,
            iv,
        })
    }

    pub fn accept(mut stream: T) -> Result<Self> {
        let mut pk_bytes = [0; crate::key_exchange::public_key_bytes()];
        stream.read_exact(&mut pk_bytes)?;
        let pk = crate::key_exchange::PublicKey::from_bytes(&pk_bytes)?;
        let (ss, ct) = crate::key_exchange::encapsulate(&pk);
        stream.write_all(ct.as_bytes())?;
        let cipher = cipher();
        let digest = MessageDigest::shake_128();
        let iv = hash(digest, ss.as_bytes())?.to_vec(); // Create iv from shared secret
        Ok(Self {
            stream,
            shared_secret: ss,
            cipher,
            iv,
        })
    }

    /// Receive and verify signature
    pub fn verify(&mut self, pk: &crate::sign::PublicKey) -> Result<()> {
        let sig_bytes = self.receive_bytes()?;
        let sig = crate::sign::DetachedSignature::from_bytes(&sig_bytes)?;
        crate::sign::verify_detached_signature(&sig, self.shared_secret.as_bytes(), &pk)?;
        Ok(())
    }

    /// Send signature
    pub fn authorize(&mut self, sk: &crate::sign::SecretKey) -> Result<()> {
        let sig = crate::sign::detached_sign(self.shared_secret.as_bytes(), sk);
        self.send_bytes(sig.as_bytes())?;
        Ok(())
    }

    pub fn send_bytes(&mut self, data: &[u8]) -> Result<()> {
        let mut size_buf = Vec::new();
        for i in 0..16-data.len().to_string().len() {
            size_buf.push(0u8)
        }
        for i in 0..data.len().to_string().len() {
            size_buf.push(data.len().to_string().chars().nth(i).unwrap() as u8);
        }

        let encrypted_bytes_len = encrypt(
            self.cipher,
            self.shared_secret.as_bytes(),
            Some(&self.iv),
            &size_buf,
        )?;

        self.stream.write_all(&encrypted_bytes_len)?;

        let encrypted = encrypt(
            self.cipher,
            self.shared_secret.as_bytes(),
            Some(&self.iv),
            data,
        )?;
        self.stream.write_all(&encrypted)?;
        Ok(())
    }

    pub fn receive_bytes(&mut self) -> Result<Vec<u8>> {
        let mut size_buf = [0; 32];
        self.stream.read_exact(&mut size_buf)?;
        let decrypted_size = decrypt(
            self.cipher,
            self.shared_secret.as_bytes(),
            Some(&self.iv),
            &size_buf,
        )?;

        let mut size: usize = 0;
        for i in 0..decrypted_size.len() {
            let a = u8::from_be_bytes(decrypted_size[i].to_be_bytes());
            if a >= b'0' && a <= b'9' {
                size = size.checked_mul(10).unwrap().checked_add((a - b'0') as usize).unwrap();
            }
        }

        size += 16 - size % 16;
        let mut buf = vec![0; size];
        self.stream.read_exact(&mut buf)?;
        let decrypted = decrypt(
            self.cipher,
            self.shared_secret.as_bytes(),
            Some(&self.iv),
            &buf,
        )?;
        Ok(decrypted)
    }

    pub fn send_json(&mut self, input: Value) -> Result<()> {
        let string = input.to_string();
        let data = string.as_bytes();

        if let Err(why) = self.send_bytes(data) {
            println!("Error: {why}");
        }

        Ok(())
    }

    pub fn receive_json(&mut self) -> Result<Value> {
        let res = match self.receive_bytes() {
            Ok(res) => String::from_utf8_lossy(&res).to_string(),
            Err(why) => return Err(why),
        };

        let received = serde_json::from_str(&res).unwrap();
        Ok(received)
    }
}
