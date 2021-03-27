use std::fs::File;
use serde::Deserialize;
use serde::Serialize;

use streams_core::{
    channel::tangle_channel::Channel,
    utility::time_utility::{current_time, TimeUnit}
};
use streams_core::users::author_builder::AuthorBuilder;
use streams_core::utility::iota_utility::create_send_options;
use streams_core::users::subscriber_builder::SubscriberBuilder;
use iota_streams::app_channels::api::tangle::Address;
use anyhow::Result;
use streams_core::payload::payload_serializer::json::Payload;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub device: String,
    pub operator: String,
    pub timestamp: u128,
    pub payload: MessagePayload
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessagePayload {
    pub temperature: f32,
    pub humidity: f32,
    pub weight: f32
}


fn get_message(device_id: &str) -> Message{
    let fr = File::open("example/message.json").unwrap();
    let mut data: Message = serde_json::from_reader(fr).unwrap();
    data.timestamp = current_time(TimeUnit::SECONDS).unwrap();
    data.device = device_id.to_string();
    data
}


fn test_channel_create() -> Result<(String, String, String)>{
    let data: Message = get_message("DEVICE_1");

    let send_opt = create_send_options(9, false);
    let node_url = "https://api.lb-0.testnet.chrysalis2.com";

    let author = AuthorBuilder::new()
        .send_options(send_opt)
        .node(node_url)
        .build()
        .unwrap();

    let mut channel: Channel = Channel::new(author);
    let (channel_address, announce_id) = channel.open().unwrap();

    println!("Channel Address: {}", &channel_address);
    println!("MsgId: {}", &announce_id);
    println!("Sending message ...");

    let msg_id = channel.write_signed(&data).unwrap();
    println!("... Message sent:");
    println!("id -> {}", msg_id);
    println!("data -> {:?}", &data);

    channel.export_to_file("mypsw", "example/channel_state.json");
    Ok((channel_address, announce_id, msg_id))
}

fn test_restore_channel(){
    let send_opt = create_send_options(9, false);
    let node = "https://api.lb-0.testnet.chrysalis2.com";

    println!("\n\nRestoring Channel ...");

    let (_, mut channel) = Channel::import_from_file(
        "example/channel_state.json",
        "mypsw",
        Some(node),
        Some(send_opt)
    ).unwrap();

    println!("... Channel Restored");

    let data = get_message("DEVICE_2");
    let channel_address = channel.channel_address();
    println!("Channel Address: {}", &channel_address);
    println!("Sending message ...");

    let msg_id = channel.write_signed(&data).unwrap();
    println!("... Message sent:");
    println!("id -> {}", msg_id);
    println!("data -> {:?}", &data);
}

pub fn test_get_messages_from_tangle(channel_address: &str, msg_id: &str, msg_id2: &str) -> Result<()>{
    let send_opts = create_send_options(9, false);
    let mut subscriber = SubscriberBuilder::new().send_options(send_opts).build().unwrap();
    let link = Address::from_str(channel_address, msg_id).unwrap();
    println!("Receiving announce");
    subscriber.receive_announcement(&link)?;
    println!("Announce received");

    let msg_link = Address::from_str(channel_address, msg_id2).unwrap();
    let (_, public_payload, _) = subscriber.receive_signed_packet(&msg_link)?;
    let p_data: Message = Payload::unwrap_data(public_payload.0).unwrap();
    println!("{:?}", p_data);
    Ok(())
}

#[tokio::main]
async fn main() {
    let (channel_address, announce_id, msg_id) = test_channel_create().unwrap();
    test_restore_channel();
    test_get_messages_from_tangle(&channel_address, &announce_id, &msg_id).unwrap();
}
