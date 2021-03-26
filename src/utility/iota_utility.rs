extern crate rand;
use anyhow::Result;
use crypto::hashes::{blake2b, Digest};
use iota_streams::app::transport::tangle::client::SendOptions;
use iota_streams::core::prelude::hex;
use rand::Rng;

///
/// Generates a new random String of 81 Chars of A..Z and 9
///
pub fn random_seed() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    const SEED_LEN: usize = 81;
    let mut rng = rand::thread_rng();

    let seed: String = (0..SEED_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
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
