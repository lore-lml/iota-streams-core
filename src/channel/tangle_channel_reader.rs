use std::string::ToString;

use anyhow::Result;
use iota_streams::{
    app::transport::tangle::client::Client as StreamsClient,
    app_channels::api::tangle::Subscriber
};
use iota_streams::app::message::HasLink;
use iota_streams::app_channels::api::tangle::MessageContent;

use crate::payload::payload_types::{StreamsPacket, StreamsPacketSerializer};
use crate::utility::iota_utility::create_link;

///
/// Channel Reader
///
pub struct ChannelReader {
    subscriber: Subscriber<StreamsClient>,
    channel_address: String,
    announcement_id: String,
    unread_msgs: Vec<(String, Vec<u8>, Vec<u8>)>,
}

impl ChannelReader {
    ///
    /// Initialize the Channel Reader
    ///
    pub fn new(subscriber: Subscriber<StreamsClient>, channel_address: &str, announcement_id: &str) -> ChannelReader {
        ChannelReader {
            subscriber,
            channel_address: channel_address.to_string(),
            announcement_id: announcement_id.to_string(),
            unread_msgs: Vec::new()
        }
    }

    ///
    /// Attach the Reader to Channel
    ///
    pub async fn attach(&mut self) -> Result<()> {
        let link = create_link(&self.channel_address, &self.announcement_id)?;
        self.subscriber.receive_announcement(&link).await
    }

    ///
    /// Send a subscription to a channel for future private communications
    ///
    pub async fn send_subscription(&mut self) -> Result<String>{
        let link = create_link(&self.channel_address, &self.announcement_id)?;
        let addr = self.subscriber.send_subscribe(&link).await?.msgid.to_string();
        Ok(addr)
    }

    ///
    /// Receive a signed packet and return it in a StreamsPacket struct that is able to parse its content to your own types
    ///
    pub async fn receive_parsed_packet<T>(&mut self, msg_id: &str, key_nonce: Option<(Vec<u8>, Vec<u8>)>) -> Result<StreamsPacket<T>>
        where
            T: StreamsPacketSerializer,
    {
        let msg_link = create_link(&self.channel_address, msg_id)?;
        let (_, public_payload, masked_payload) = self.subscriber.receive_signed_packet(&msg_link).await?;
        let (p_data, m_data) = (&public_payload.0, &masked_payload.0);

        StreamsPacket::from_streams_response(&p_data, &m_data, &key_nonce)
    }

    ///
    /// Receive a signed packet in raw format. It returns a tuple (pub_bytes, masked_bytes)
    ///
    pub async fn receive_raw_packet<T>(&mut self, msg_id: &str) -> Result<(Vec<u8>, Vec<u8>)>
        where
            T: StreamsPacketSerializer,
    {
        let msg_link = create_link(&self.channel_address, msg_id)?;
        let (_, public_payload, masked_payload) = self.subscriber.receive_signed_packet(&msg_link).await?;
        Ok((public_payload.0.clone(), masked_payload.0.clone()))
    }

    ///
    /// Fetch all the remaining msgs
    ///
    /// # Return Value
    /// It returns a Vector of Tuple containing (msg_id, public_bytes, masked_bytes)
    ///
    pub async fn fetch_raw_msgs(&mut self) -> Vec<(String, Vec<u8>, Vec<u8>)> {
        while self.fetch_next_msgs().await > 0 {};
        self.unread_msgs.clone()
    }

    ///
    /// Fetch all the remaining msgs
    ///
    /// # Return Value
    /// It returns a Vector of StreamsPacket that can parse its content
    ///
    pub async fn fetch_parsed_msgs<T>(&mut self, key_nonce: &Option<(Vec<u8>, Vec<u8>)>) -> Result<Vec<(String, StreamsPacket<T>)>>
    where
        T: StreamsPacketSerializer
    {
        while self.fetch_next_msgs().await > 0 {};

        let mut res = vec![];
        for (id, p, m) in &self.unread_msgs {
            res.push((id.clone(), StreamsPacket::from_streams_response(p, m, key_nonce)?));
        }

        Ok(res)
    }

    ///
    /// Get the channel address and the announcement id
    ///
    pub fn channel_address(&self) -> (String, String){
        (self.channel_address.clone(), self.announcement_id.clone())
    }
}

impl ChannelReader{
    pub async fn fetch_next_msgs(&mut self) -> u8{
        let mut found = 0;
        let msgs = self.subscriber.fetch_next_msgs().await;
        for msg in msgs {
            let link = msg.link.rel();
            match msg.body{
                MessageContent::SignedPacket {pk: _, public_payload, masked_payload } => {
                    found += 1;
                    let p = public_payload.0;
                    let m = masked_payload.0;


                    /*println!("Packet found:");
                    println!("Public: {}", hex::encode(&p));
                    println!("Masked: {}", hex::encode(&m));*/

                    if !p.is_empty() || !m.is_empty(){
                        self.unread_msgs.push((link.to_string(), p, m));
                    }
                }
                _ => {println!("{}", link.to_string());}
            }
        }
        found
    }
}
