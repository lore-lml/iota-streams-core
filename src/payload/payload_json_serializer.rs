use serde::{de::DeserializeOwned, Serialize};
use crate::payload::payload_types::{StreamsPacket, StreamsPacketBuilder, StreamsPayloadSerializer};
use anyhow::Result;

///
/// Implementation of JSON Serialize
///
pub struct JsonSerializer;

impl StreamsPayloadSerializer for JsonSerializer {
    fn serialize<T: Serialize>(data: &T) -> Result<String> {
        serde_json::to_string(data).map_err(|e| anyhow::anyhow!(e))
    }

    fn deserialize<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
        serde_json::from_slice(data).map_err(|e| anyhow::anyhow!(e))
    }
}

/// Payload JSON
///
pub type JsonPacket = StreamsPacket<JsonSerializer>;

/// Payload Builder in Json Format
///
pub type JsonPacketBuilder = StreamsPacketBuilder<JsonSerializer>;


