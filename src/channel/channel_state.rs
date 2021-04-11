use std::fs::OpenOptions;
use std::io::{Read, Write};

use aead::generic_array::GenericArray;
use anyhow::Result;
use base64::{decode_config, encode_config, URL_SAFE_NO_PAD};
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::XChaCha20Poly1305;
use serde::{Deserialize, Serialize};

use crate::utility::iota_utility::hash_string;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelState{
    author_state: Vec<u8>,
    announcement_id: String,
    last_msg_id: String,
}

impl ChannelState {
    pub fn new(author_state: &Vec<u8>, announcement_id: &str, last_public_msg: &str) -> ChannelState{
        ChannelState{
            author_state: author_state.clone(),
            announcement_id: announcement_id.to_string(),
            last_msg_id: last_public_msg.to_string(),
        }
    }

    pub fn from_file(file_path: &str, psw: &str) -> Result<ChannelState>{
        let mut fr = OpenOptions::new().read(true).open(file_path).unwrap();
        let mut input = vec![];
        fr.read_to_end(&mut input)?;
        let ch_state = ChannelState::decrypt(&input, psw)?;
        println!("\n{:?}\n", ch_state);
        Ok(ch_state)
    }
}

impl ChannelState{
    pub fn write_to_file(&self, file_path: &str, psw: &str) -> Result<()>{
        let mut fr = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file_path)?;
        println!("\n{:?}\n", &self);
        fr.write_all(&self.encrypt(psw)?.as_bytes())?;
        Ok(())
    }

    pub fn author_state(&self) -> Vec<u8> {
        self.author_state.clone()
    }
    pub fn announcement_id(&self) -> String {
        self.announcement_id.clone()
    }
    pub fn last_msg_id(&self) -> String {
        self.last_msg_id.clone()
    }
}

impl ChannelState{
    fn encrypt(&self, psw: &str) -> Result<String>{
        let bytes = bincode::serialize(&self)?;

        let (key, nonce) = get_key_nonce(psw);
        let key = GenericArray::from_slice(&key[..]);
        let nonce = GenericArray::from_slice(&nonce[..]);

        let chacha = XChaCha20Poly1305::new(key);
        let enc = match chacha.encrypt(nonce, bytes.as_ref()){
            Ok(res) => res,
            Err(_) => return Err(anyhow::Error::msg("Error during state encryption")),
        };
        let base64 = encode_config(&enc, URL_SAFE_NO_PAD);
        Ok(base64)
    }

    fn decrypt(input: &[u8], psw: &str) -> Result<ChannelState>{
        let bytes = decode_config(input, URL_SAFE_NO_PAD)?;

        let (key, nonce) = get_key_nonce(psw);
        let key = GenericArray::from_slice(&key[..]);
        let nonce = GenericArray::from_slice(&nonce[..]);

        let chacha = XChaCha20Poly1305::new(key);
        let dec = match chacha.decrypt(nonce, bytes.as_ref()){
            Ok(res) => res,
            Err(_) => return Err(anyhow::Error::msg("Error during state decryption")),
        };

        let ch_state: ChannelState = bincode::deserialize(&dec)?;
        Ok(ch_state)
    }
}

fn get_key_nonce(psw: &str) -> (Vec<u8>, Vec<u8>) {
    let key_hash = &hash_string(psw)[..32];
    let nonce_hash = &hash_string(key_hash)[..24];
    let key = key_hash.as_bytes();
    let nonce = nonce_hash.as_bytes();
    (key.to_vec(), nonce.to_vec())
}
