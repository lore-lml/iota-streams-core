use anyhow::Result;
use iota_streams::core::prelude::hex;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::payload::payload_types::{StreamsPacket, StreamsPacketBuilder, StreamsPacketSerializer};

pub struct RawSerializer;

impl StreamsPacketSerializer for RawSerializer{
    fn serialize<T: Serialize>(data: &T) -> Result<String> {
        let bytes = bincode::serialize(data)?;
        Ok(hex::encode(bytes))
    }

    fn deserialize<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
        let data = hex::decode(data)?;
        Ok(bincode::deserialize(&data)?)
    }
}

pub type Packet = StreamsPacket<RawSerializer>;
pub type PacketBuilder = StreamsPacketBuilder<RawSerializer>;
