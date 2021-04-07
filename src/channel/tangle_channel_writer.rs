use crate::payload::payload_serializer::{empty_bytes, json::PayloadBuilder, PacketPayload};

use anyhow::{Result, Error};
use iota_streams::{
    app::transport::tangle::client::{Client as StreamsClient, SendOptions},
    app_channels::api::tangle::{Author, Address},
};

use std::string::ToString;
use crate::channel::channel_state::ChannelState;
use crate::utility::iota_utility::{hash_string, create_link};
use crate::user_builders::author_builder::AuthorBuilder;
use iota_streams::ddml::types::Bytes;

///
/// Channel
///
pub struct ChannelWriter {
    author: Author<StreamsClient>,
    channel_address: String,
    announcement_id: String,
    prev_public_msg: String,
    prev_masked_msg: String,
}

impl ChannelWriter {
    ///
    /// Initialize the Channel
    ///
    pub fn new(author: Author<StreamsClient>) -> ChannelWriter {
        let channel_address = author.channel_address().unwrap().to_string();
        ChannelWriter {
            author,
            channel_address,
            announcement_id: String::default(),
            prev_public_msg: String::default(),
            prev_masked_msg: String::default(),
        }
    }

    fn import(channel_state: &ChannelState, psw: &str, node_url: Option<&str>, send_options: Option<SendOptions>) -> Result<ChannelWriter>{
        let author = AuthorBuilder::build_from_state(
            &channel_state.author_state(),
            psw,
            node_url,
            send_options
        )?;
        let channel_address = author.channel_address().unwrap().to_string();

        Ok(ChannelWriter {
            author,
            channel_address,
            announcement_id: channel_state.announcement_id(),
            prev_public_msg: channel_state.last_public_msg(),
            prev_masked_msg: channel_state.last_masked_msg(),
        })
    }

    ///
    /// Restore the channel from a previously stored state in a file
    ///
    pub fn import_from_file(file_path: &str, psw: &str, node_url: Option<&str>, send_options: Option<SendOptions>) -> Result<(ChannelState, ChannelWriter)>{
        let channel_state = ChannelState::from_file(file_path)?;
        let channel = ChannelWriter::import(&channel_state, psw, node_url, send_options)?;
        Ok((channel_state, channel))
    }

    ///
    /// Open a channel
    ///
    pub async fn open(&mut self) -> Result<(String, String)> {
        let announce = self.author.send_announce().await?;
        self.announcement_id = announce.msgid.to_string();
        self.prev_public_msg = self.announcement_id.clone();
        Ok((self.channel_address.clone(), self.announcement_id.clone()))
    }

    ///
    /// Receive a single subscription message
    ///
    pub async fn get_single_sub(&mut self, sub_addr: String) -> Result<bool>{
        let link = create_link(&self.channel_address, &sub_addr)?;
        Ok(self.author.receive_subscribe(&link).await.is_ok())
    }

    ///
    /// Receive multiple subscription message
    ///
    pub async fn get_multiple_subs(&mut self, sub_ids: Vec<String>) -> Result<Vec<bool>>{
        let mut res: Vec<bool> = Vec::new();
        for id in sub_ids {
            res.push(self.get_single_sub(id).await?);
        }
        Ok(res)
    }

    ///
    /// It sends a keyload message in order to start private communications among approved subscribers
    ///
    pub async fn start_private_communications(&mut self) -> Result<String>{
        if self.announcement_id == String::default(){
            return Err(Error::msg("You must open the channel first"))
        }
        let announce = create_link(&self.channel_address, &self.announcement_id)?;
        let (keyload, _) = self.author.send_keyload_for_everyone(&announce).await?;
        let keyload_id = keyload.msgid;
        self.prev_masked_msg = keyload_id.to_string();
        Ok(keyload_id.to_string())
    }

    ///
    /// Write signed packet with formatted data.
    ///
    pub async fn send_signed_json<T>(&mut self, data: T, is_masked: bool) -> Result<String>
        where
            T: serde::Serialize
    {
        let link_to = self.get_next_link(is_masked)?;
        let (public_payload, masked_payload) = ChannelWriter::get_payloads(data, is_masked);

        let ret_link = self.author.send_signed_packet(
            &link_to,
            &public_payload,
            &masked_payload,
        ).await?;

        let msg_id = ret_link.0.msgid.to_string();
        if !is_masked{
            self.prev_public_msg = msg_id.clone();
        }else{
            self.prev_masked_msg = msg_id.clone();
        }

        Ok(msg_id)
    }

    ///
    /// Write signed packet with raw data.
    ///
    pub async fn send_signed_raw_data(&mut self, data: Vec<u8>, is_masked: bool) -> Result<String>
    {
        let link_to = self.get_next_link(is_masked)?;
        let (public_payload, masked_payload) = {
          if is_masked{
              (empty_bytes(), Bytes(data))
          }else {
              (Bytes(data), empty_bytes())
          }
        };

        let ret_link = self.author.send_signed_packet(
            &link_to,
            &public_payload,
            &masked_payload,
        ).await?;

        let msg_id = ret_link.0.msgid.to_string();
        if !is_masked{
            self.prev_public_msg = msg_id.clone();
        }else{
            self.prev_masked_msg = msg_id.clone();
        }

        Ok(msg_id)
    }


    fn export(&self, psw: &str) -> Result<ChannelState>{
        let psw_hash = hash_string(psw)?;
        let author_state = self.author.export(&psw_hash)?;
        Ok(ChannelState::new(&author_state, &self.announcement_id, &self.prev_public_msg, &self.prev_masked_msg))
    }

    ///
    /// Stores the channel state in a file. The author state is encrypted with the specified password
    ///
    pub fn export_to_file(&self, psw: &str, file_path: &str)-> Result<()>{
        let channel_state = self.export(psw)?;
        channel_state.write_to_file(file_path);
        Ok(())
    }

    ///
    /// Get the channel address and the announcement id
    ///
    pub fn channel_address(&self) -> (String, String){
        (self.channel_address.clone(), self.announcement_id.clone())
    }
}

impl ChannelWriter{
    fn get_payloads<T>(data: T, is_masked: bool) -> (Bytes, Bytes)
    where
        T: serde::Serialize
    {
        match is_masked {
            true => (empty_bytes(), PayloadBuilder::new().masked(&data).unwrap().build().masked_data().clone()),
            false => (PayloadBuilder::new().public(&data).unwrap().build().public_data().clone(), empty_bytes())
        }
    }

    fn get_next_link(&self, is_masked: bool) -> Result<Address> {
        let (c1, c2) = (self.prev_public_msg == String::default(), self.prev_masked_msg == String::default());
        match (c1, c2, is_masked){
            (true, _, _) => Err(Error::msg("You forgot to open the channel")),
            (false, true, true) => Err(Error::msg("You forgot to start a private communications")),
            (false, false, true) => create_link(&self.channel_address, &self.prev_masked_msg),
            (false, _, false) => create_link(&self.channel_address, &self.prev_public_msg),
        }
    }
}
