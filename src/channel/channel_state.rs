use std::fs::OpenOptions;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use iota_streams::core::prelude::hex;
use crate::utility::iota_utility::hash_string;
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::aead::{NewAead, Aead};

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

    pub fn from_file(file_path: &str) -> Result<ChannelState>{
        let fr = OpenOptions::new().read(true).open(file_path).unwrap();
        let channel_state: ChannelState = serde_json::from_reader(fr).unwrap();
        Ok(channel_state)
    }
}

impl ChannelState{
    pub fn write_to_file(&self, file_path: &str) -> Result<()>{
        let fr = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file_path)?;
        serde_json::to_writer(fr, &self)?;
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
    fn self_encrypt(&self, psw: &str) -> Result<Vec<u8>>{
        let bytes = bincode::serialize(&self)?;
        let key_hash = &hash_string(psw)[..32];
        let nonce_hash = &hash_string(key_hash)[..24];
        let key = key_hash.as_bytes();
        let nonce = nonce_hash.as_bytes();

        let chacha = XChaCha20Poly1305::new(key);
        let enc = chacha.encrypt(nonce, bytes.as_ref());
        let hex = hex::encode(&enc);
        Ok(Vec::default())
    }
}
