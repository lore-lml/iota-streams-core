extern crate rand;
use anyhow::{Result, bail};
use crypto::hashes::{blake2b, Digest};
use iota_streams::app::transport::tangle::client::{SendOptions, Client};
use iota_streams::core::prelude::hex;
use rand::Rng;
use iota_streams::app_channels::api::tangle::{Address, Author};
use std::fs::OpenOptions;
use std::io::{Write, Read};
use iota_streams::app::transport::TransportOptions;

///
/// Generates a new random String of 81 Chars of A..Z and 9
///
pub fn random_seed() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    const SEED_LEN: usize = 81;
    let mut rng = rand::thread_rng();

    let seed: String = (0..SEED_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    seed
}

///
/// Generates SendOptions struct with the specified mwm and pow
///
pub fn create_send_options(min_weight_magnitude: u8, local_pow: bool) -> SendOptions{
    let mut send_opt = SendOptions::default();
    send_opt.min_weight_magnitude = min_weight_magnitude;
    send_opt.local_pow = local_pow;
    send_opt
}

pub fn hash_string(string: &str) ->  Result<String>  {
    let hash = blake2b::Blake2b256::digest(&string.as_bytes());
    Ok(hex::encode(&hash))
}

pub fn create_link(channel_address: &str, msg_id: &str) -> Result<Address>{
    match Address::from_str(channel_address, msg_id) {
        Ok(link) => Ok(link),
        Err(()) => bail!("Failed to create Address from {}:{}", channel_address, msg_id)
    }
}

pub fn export_author_state(author: &Author<Client>, path: &str, psw: &str) -> Result<()>{
    let mut fr = OpenOptions::new().write(true).create(true).truncate(true).open(path)?;
    let state = author.export(psw)?;
    fr.write_all(&state)?;
    Ok(())
}

pub fn import_author_from_state(path: &str, psw: &str) -> Result<Author<Client>>{
    let opts = create_send_options(9, false);
    let mut client = Client::new_from_url("https://api.lb-0.testnet.chrysalis2.com");
    client.set_send_options(opts);
    let mut fr = OpenOptions::new().read(true).open(path)?;
    let mut state: Vec<u8> = Vec::new();
    fr.read_to_end(&mut state)?;
    Author::import(&state, psw, client)
}
