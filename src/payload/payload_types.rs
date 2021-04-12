use std::marker::PhantomData;

use anyhow::{Error, Result};
use base64::{decode_config, encode_config, URL_SAFE_NO_PAD};
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::XChaCha20Poly1305;
use iota_streams::ddml::types::Bytes;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait StreamsPacketSerializer {
    fn serialize<T: Serialize>(data: &T) -> Result<String>;
    fn deserialize<T: DeserializeOwned>(data: &[u8]) -> Result<T>;
}

pub struct StreamsPacket<P>{
    p_data: Vec<u8>,
    m_data: Vec<u8>,
    _marker: PhantomData<P>,
    key_nonce: Option<([u8;32], [u8;24])>,
}

impl<P> StreamsPacket<P>
where
    P: StreamsPacketSerializer,
{
    fn new(p_data: &[u8], m_data: &[u8], key_nonce: Option<([u8;32], [u8;24])>) -> StreamsPacket<P>{
        StreamsPacket{
            p_data: p_data.to_vec(),
            m_data: m_data.to_vec(),
            _marker: PhantomData,
            key_nonce
        }
    }

    pub fn from_streams_response(p_data: &[u8], m_data: &[u8], key_nonce: &Option<([u8;32], [u8;24])>) -> Result<StreamsPacket<P>>{
        let (p, m) = match key_nonce{
            None => (p_data.to_vec(), m_data.to_vec()),
            Some((key, nonce)) => {
                let key_arr = GenericArray::from_slice(&key[..]);
                let nonce_arr = GenericArray::from_slice(&nonce[..]);
                let chacha = XChaCha20Poly1305::new(key_arr);
                match chacha.decrypt(nonce_arr, m_data.as_ref()){
                    Ok(dec) => (p_data.to_vec(), dec),
                    Err(_) => return Err(Error::msg("Error during data decryption"))
                }
            }
        };

        Ok(
            StreamsPacket{
            p_data: decode_config(p, URL_SAFE_NO_PAD)?,
            m_data: decode_config(m, URL_SAFE_NO_PAD)?,
            _marker: PhantomData,
            key_nonce: key_nonce.clone(),
            }
        )
    }

    pub fn public_data(&self) -> Result<Bytes> {
        let p = encode_config(&self.p_data, URL_SAFE_NO_PAD).as_bytes().to_vec();
        Ok(Bytes(p))
    }

    pub fn masked_data(&self) -> Result<Bytes> {
        let m = encode_config(&self.m_data, URL_SAFE_NO_PAD).as_bytes().to_vec();
        let data = match &self.key_nonce{
            None => m,
            Some((key, nonce)) => {
                let key_arr = GenericArray::from_slice(&key[..]);
                let nonce_arr = GenericArray::from_slice(&nonce[..]);
                let chacha = XChaCha20Poly1305::new(key_arr);
                match chacha.encrypt(nonce_arr, m.as_ref()){
                    Ok(enc) => enc,
                    Err(_) => return Err(Error::msg("Error during data encryption"))
                }
            }
        };

        Ok(Bytes(data))
    }

    pub fn parse_data<U, T>(&self) -> Result<(U, T)>
    where
        U: DeserializeOwned,
        T: DeserializeOwned,
    {
        Ok((P::deserialize(&self.p_data)?, P::deserialize(&self.m_data)?))
    }

}

pub struct StreamsPacketBuilder<P>{
    public: String,
    masked: String,
    _pub_marker: PhantomData<P>,
    key_nonce: Option<([u8;32], [u8;24])>,
}

impl<P> StreamsPacketBuilder<P>
where
    P: StreamsPacketSerializer,
{
    pub fn new() -> Self{
        StreamsPacketBuilder{
            public: String::new(),
            masked: String::new(),
            _pub_marker: PhantomData,
            key_nonce: None
        }
    }

    pub fn public<T>(&mut self, data: &T) -> Result<&mut Self>
    where
        T: serde::Serialize{
        self.public = P::serialize(data)?;
        Ok(self)
    }

    pub fn masked<T>(&mut self, data: &T) -> Result<&mut Self>
    where
        T: serde::Serialize
    {
        self.masked = P::serialize(data)?;
        Ok(self)
    }

    pub fn key_nonce(&mut self, key: &[u8;32], nonce: &[u8;24]) -> &mut Self{
        self.key_nonce = Some((key.clone(), nonce.clone()));
        self
    }

    pub fn build(&mut self) -> StreamsPacket<P> {
        StreamsPacket::new(&self.public.as_bytes(), &self.masked.as_bytes(), self.key_nonce.clone())
    }

}
