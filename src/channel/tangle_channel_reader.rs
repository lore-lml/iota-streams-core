use anyhow::Result;
use iota_streams::{
    app::transport::tangle::client::{Client as StreamsClient},
    app_channels::api::tangle::Subscriber
};

use std::string::ToString;
use crate::payload::payload_serializer::json::Payload;
use serde::de::DeserializeOwned;
use iota_streams::app_channels::api::tangle::MessageContent;
use iota_streams::app::message::HasLink;
use crate::utility::iota_utility::create_link;

///
/// Channel Reader
///
pub struct ChannelReader {
    subscriber: Subscriber<StreamsClient>,
    channel_address: String,
    announcement_id: String,
    unread_msgs: Vec<(String, Vec<u8>, bool)>,
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
    /// Join to the private communications of the specified keyload address
    ///
    #[allow(dead_code)]
    async fn join_private_communications(&mut self, address: String) -> Result<bool>{
        let link = create_link(&self.channel_address, &address)?;
        self.subscriber.receive_keyload(&link).await
    }

    ///
    /// Receive a signed packet and parse its content in a struct with the`DeserializeOwned`trait
    ///
    pub async fn receive_signed<T>(&mut self, msg_id: &str) -> Result<T>
        where
            T: DeserializeOwned
    {
        let msg_link = create_link(&self.channel_address, msg_id)?;
        let (_, public_payload, _) = self.subscriber.receive_signed_packet(&msg_link).await?;
        let p_data: T = Payload::unwrap_data(public_payload.0).unwrap();
        Ok(p_data)
    }

    ///
    /// Fetch all the remaining msgs
    ///
    /// # Return Value
    /// It returns a Vector of Tuple containing (msg_id, content_bytes)
    ///
    pub async fn get_all_msgs(&mut self) -> Vec<(String, Vec<u8>, bool)> {
        while self.fetch_next_msgs().await > 0 {};
        self.unread_msgs.clone()
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
                MessageContent::Keyload => {
                    found += 1;
                    println!("Keyload found");
                }
                MessageContent::SignedPacket {pk: _, public_payload, masked_payload } => {
                    found += 1;
                    let p = public_payload.0;
                    let m = masked_payload.0;


                    println!("Packet found:");
                    println!("Public: {}", String::from_utf8(p.clone()).unwrap());
                    println!("Masked: {}", String::from_utf8(m.clone()).unwrap());

                    if p.len() > 0 {
                        self.unread_msgs.push((link.to_string(), p, false));
                    }else if m.len() > 0{
                        self.unread_msgs.push((link.to_string(), m, true));
                    }
                }
                _ => {println!("{}", link.to_string());}
            }
        }
        found
    }
}
