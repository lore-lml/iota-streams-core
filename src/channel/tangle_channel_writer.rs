use std::string::ToString;

use anyhow::Result;
use iota_streams::{
    app::transport::tangle::client::{Client as StreamsClient, SendOptions},
    app_channels::api::tangle::Author,
};

use crate::channel::channel_state::ChannelState;
use crate::payload::payload_raw_serializer::PacketBuilder;
use crate::payload::payload_types::{StreamsPacket, StreamsPacketSerializer};
use crate::user_builders::author_builder::AuthorBuilder;
use crate::utility::iota_utility::{create_link, hash_string};

///
/// Channel
///
pub struct ChannelWriter {
    author: Author<StreamsClient>,
    channel_address: String,
    announcement_id: String,
    last_msg_id: String
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
            last_msg_id: String::default(),
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
            last_msg_id: channel_state.last_msg_id(),
        })
    }

    ///
    /// Restore the channel from a previously stored state in a file
    ///
    pub fn import_from_file(file_path: &str, psw: &str, node_url: Option<&str>, send_options: Option<SendOptions>) -> Result<(ChannelState, ChannelWriter)>{
        let channel_state = ChannelState::from_file(file_path, &psw)?;
        let channel = ChannelWriter::import(&channel_state, psw, node_url, send_options)?;
        Ok((channel_state, channel))
    }

    ///
    /// Open a channel
    ///
    pub async fn open(&mut self) -> Result<(String, String)> {
        let announce = self.author.send_announce().await?;
        self.announcement_id = announce.msgid.to_string();
        self.last_msg_id = self.announcement_id.clone();
        Ok((self.channel_address.clone(), self.announcement_id.clone()))
    }

    ///
    /// Write signed packet with formatted data.
    ///
    pub async fn send_signed_raw_data(&mut self, p_data: Vec<u8>, m_data: Vec<u8>, key_nonce: Option<(Vec<u8>, Vec<u8>)>) -> Result<String> {
        let link_to = create_link(&self.channel_address, &self.last_msg_id)?;
        let packet = match key_nonce{
            None => PacketBuilder::new()
                .public(&p_data)?
                .masked(&m_data)?
                .build(),
            Some((key, nonce)) => PacketBuilder::new()
                .public(&p_data)?
                .masked(&m_data)?
                .key_nonce(&key, &nonce)
                .build()
        };

        let ret_link = self.author.send_signed_packet(
            &link_to,
            &packet.public_data()?,
            &packet.masked_data()?,
        ).await?;

        let msg_id = ret_link.0.msgid.to_string();
        self.last_msg_id = msg_id.clone();
        Ok(msg_id)
    }

    ///
    /// Write signed packet with formatted data.
    ///
    pub async fn send_signed_packet<T>(&mut self, packet: &StreamsPacket<T>) -> Result<String>
    where
        T: StreamsPacketSerializer,
    {
        let link_to = create_link(&self.channel_address, &self.last_msg_id)?;
        let (public_payload, masked_payload) = (packet.public_data()?, packet.masked_data()?);

        let ret_link = self.author.send_signed_packet(
            &link_to,
            &public_payload,
            &masked_payload,
        ).await?;

        let msg_id = ret_link.0.msgid.to_string();
        self.last_msg_id = msg_id.clone();
        Ok(msg_id)
    }


    fn export(&self, psw: &str) -> Result<ChannelState>{
        let psw_hash = hash_string(psw);
        let author_state = self.author.export(&psw_hash)?;
        Ok(ChannelState::new(&author_state, &self.announcement_id, &self.last_msg_id))
    }

    ///
    /// Stores the channel state in a file. The author state is encrypted with the specified password
    ///
    pub fn export_to_file(&self, psw: &str, file_path: &str)-> Result<()>{
        let channel_state = self.export(psw)?;
        channel_state.write_to_file(file_path, psw)?;
        Ok(())
    }

    ///
    /// Get the channel address and the announcement id
    ///
    pub fn channel_address(&self) -> (String, String){
        (self.channel_address.clone(), self.announcement_id.clone())
    }
}
