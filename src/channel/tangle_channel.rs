use crate::{
    payload::payload_serializer::{empty_bytes, json::PayloadBuilder, PacketPayload},
    utility::iota_utility::random_seed,
};

use anyhow::Result;
use iota_streams::{
    app::transport::{
        TransportOptions,
        tangle::{
            client::{Client as StreamsClient, SendOptions},
            PAYLOAD_BYTES,
        },
    },
    app_channels::api::tangle::{Address, Author},
};

use std::string::ToString;

///
/// Channel
///
pub struct Channel {
    author: Author<StreamsClient>,
    channel_address: String,
    announcement_id: String,
    previous_msg_tag: String,
}

impl Channel {
    ///
    /// Initialize the Channel
    ///
    pub fn new(author: Author<StreamsClient>) -> Channel {
        let channel_address = author.channel_address().unwrap().to_string();
        Channel {
            author: author,
            channel_address: channel_address,
            announcement_id: String::default(),
            previous_msg_tag: String::default(),
        }
    }

    ///
    /// Open a channel
    ///
    pub fn open(&mut self) -> Result<(String, String)> {
        let announce = self.author.send_announce()?;
        self.announcement_id = announce.msgid.to_string();
        Ok((self.channel_address.clone(), self.announcement_id.clone()))
    }

    ///
    /// Write signed packet
    ///
    pub fn write_signed<T>(&mut self, data: T) -> Result<String>
        where
            T: serde::Serialize,
    {
        let payload = PayloadBuilder::new().public(&data).unwrap().build();
        let link_to = if self.previous_msg_tag == String::default() {
            Address::from_str(&self.channel_address, &self.announcement_id)
        }else {
            Address::from_str(&self.channel_address, &self.previous_msg_tag)
        }.unwrap();

        let ret_link = self.author.send_signed_packet(
            &link_to,
            &payload.public_data(),
            &empty_bytes(),
        )?;

        let msg_id = ret_link.0.msgid.to_string();
        self.previous_msg_tag = msg_id.clone();

        Ok(msg_id)
    }
}
