use crate::payload::payload_serializer::{empty_bytes, json::PayloadBuilder, PacketPayload};

use anyhow::Result;
use iota_streams::{
    app::transport::tangle::client::{Client as StreamsClient, SendOptions},
    app_channels::api::tangle::{Address, Author},
};

use std::string::ToString;
use crate::channel::channel_state::ChannelState;
use crate::utility::iota_utility::hash_string;
use crate::users::author_builder::AuthorBuilder;

///
/// Channel
///
pub struct Channel {
    author: Author<StreamsClient>,
    channel_address: String,
    announcement_id: String,
    previous_msg_id: String,
    //state_path: Option<String>
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
            previous_msg_id: String::default(),
        }
    }

    pub fn import(channel_state: &ChannelState, psw: &str, node_url: Option<&str>, send_options: Option<SendOptions>) -> Result<Channel>{
        let author = AuthorBuilder::build_from_state(
            &channel_state.author_state(),
            psw,
            node_url,
            send_options
        ).unwrap();
        let channel_address = author.channel_address().unwrap().to_string();
        Ok(Channel {
            author,
            channel_address,
            announcement_id: channel_state.announcement_id(),
            previous_msg_id: channel_state.last_msg_id()
        })
    }

    pub fn import_from_file(file_path: &str, psw: &str, node_url: Option<&str>, send_options: Option<SendOptions>) -> Result<(ChannelState, Channel)>{
        let channel_state = ChannelState::from_file(file_path).unwrap();
        let channel = Channel::import(&channel_state, psw, node_url, send_options).unwrap();
        Ok((channel_state, channel))
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
        let link_to = if self.previous_msg_id == String::default() {
            Address::from_str(&self.channel_address, &self.announcement_id)
        }else {
            Address::from_str(&self.channel_address, &self.previous_msg_id)
        }.unwrap();

        let ret_link = self.author.send_signed_packet(
            &link_to,
            &payload.public_data(),
            &empty_bytes(),
        )?;

        let msg_id = ret_link.0.msgid.to_string();
        self.previous_msg_id = msg_id.clone();

        Ok(msg_id)
    }

    pub fn export(&self, psw: &str) -> Result<ChannelState>{
        let psw_hash = hash_string(psw).unwrap();
        let author_state = self.author.export(&psw_hash).unwrap();
        Ok(ChannelState::new(&author_state, &self.announcement_id, &self.previous_msg_id))
    }

    pub fn export_to_file(&self, psw: &str, file_path: &str){
        let channel_state = self.export(psw).unwrap();
        channel_state.write_to_file(file_path);
    }

    pub fn channel_address(&self) -> String{
        self.channel_address.clone()
    }
}
