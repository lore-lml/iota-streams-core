use std::fs::File;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use iota_streams_lib::{
    channel::{
        tangle_channel::Channel,
        tangle_channel_reader::ChannelReader
    },
    payload::payload_serializer::json::Payload,
    users::{
        author_builder::AuthorBuilder,
        subscriber_builder::SubscriberBuilder
    },
    utility::{
        iota_utility::create_send_options,
        time_utility::{current_time, TimeUnit}
    }
};

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

fn send_signed_message(channel: &mut Channel, device_id: &str){
    println!("Sending message ...");
    let data: Message = get_message(device_id);
    let msg_id = channel.write_signed(&data).unwrap();
    println!("... Message sent:");
    println!("id -> {}", msg_id);
    println!("data -> {:?}\n\n", &data);
}


fn test_channel_create() -> Result<(String, String)>{
    let send_opt = create_send_options(9, false);
    let node_url = "https://api.lb-0.testnet.chrysalis2.com";

    let author = AuthorBuilder::new()
        .send_options(send_opt)
        .node(node_url)
        .build()
        .unwrap();

    let mut channel: Channel = Channel::new(author);
    let (channel_address, announce_id) = channel.open().unwrap();

    println!("Channel: {}:{}", &channel_address, &announce_id);

    for i in 1..=2{
        let device = format!("DEVICE_{}", i);
        send_signed_message(&mut channel, &device);
    }

    channel.export_to_file("mypsw", "example/channel_state.json")?;
    Ok((channel_address, announce_id))
}

fn test_restore_channel() -> Result<()>{
    let send_opt = create_send_options(9, false);
    let node = "https://api.lb-0.testnet.chrysalis2.com";

    println!("Restoring Channel ...");
    let (_, mut channel) = Channel::import_from_file(
        "example/channel_state.json",
        "mypsw",
        Some(node),
        Some(send_opt)
    )?;
    println!("... Channel Restored");

    let (channel_address, announce_id)= channel.channel_address();
    println!("Channel: {}:{}", &channel_address, announce_id);

    send_signed_message(&mut channel, "DEVICE_3");
    Ok(())
}

pub fn test_get_messages_from_tangle(channel_address: &str, announce_id: &str) -> Result<()>{
    let send_opts = create_send_options(9, false);
    let subscriber = SubscriberBuilder::new().send_options(send_opts).build().unwrap();

    println!("Receiving announce");
    let mut channel_reader = ChannelReader::new(subscriber, channel_address, announce_id);
    channel_reader.attach()?;
    println!("Announce received");
    println!("Channel: {}:{}\n", channel_address, announce_id);

    println!("Getting messages ... ");
    let msgs = channel_reader.fetch_remaining_msgs();
    for m in msgs {
        let address = m.0;
        let data: Message = Payload::unwrap_data(m.1).unwrap();
        println!("\nMessage {}:\n{:?}\n", address, data);
    }
    println!("... No more messages\n\n");
    Ok(())
}

fn start_test() -> Result<()>{
    let (channel_address, announce_id) = test_channel_create()?;
    test_restore_channel()?;
    test_get_messages_from_tangle(&channel_address, &announce_id)
}

#[tokio::main]
async fn main() {
    match start_test() {
        Ok(_) => println!("\n******************** Everything has worked ********************"),
        Err(_) => println!("\n******************** Something went wrong ********************")
    }
}
