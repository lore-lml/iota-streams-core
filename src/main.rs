use serde::{Deserialize, Serialize};

use iota_streams_lib::{
    utility::{
        iota_utility::create_send_options,
        time_utility::{current_time, TimeUnit}
    }
};

use std::fs::File;
use iota_streams_lib::payload::payload_json_serializer::JsonPacketBuilder;
use iota_streams_lib::user_builders::author_builder::AuthorBuilder;
use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use anyhow::Result;


#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub device: String,
    pub operator: String,
    pub timestamp: u128,
    pub payload: MessagePayload
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

async fn send_signed_message(channel: &mut ChannelWriter, device_id: &str){
    println!("Sending message ...");
    let p: Message = get_message(&format!("PUBLIC: {}", device_id));
    let m: Message = get_message(&format!("PRIVATE: {}", device_id));
    let key = b"an example very very secret key.";
    let nonce = b"extra long unique nonce!";
    let data = JsonPacketBuilder::new()
        .public(&p).unwrap()
        .masked(&m).unwrap()
        .key_nonce(key, nonce)
        .build();
    let msg_id = channel.send_signed_packet(&data).await.unwrap();
    println!("... Message sent:");
    println!("  id: {}", msg_id);
    println!("  public: {:?}", p);
    println!("  masked: {:?}\n\n", m);
}

async fn test_channel_create() -> Result<(String, String)>{
    let send_opt = create_send_options(9, false);
    let node_url = "https://api.lb-0.testnet.chrysalis2.com";

    let author = AuthorBuilder::new()
        .send_options(send_opt)
        .node(node_url)
        .build();

    let mut channel = ChannelWriter::new(author);
    let (channel_address, announce_id) = channel.open().await?;
    println!("Channel: {}:{}", &channel_address, &announce_id);

    for i in 1..=2{
        let device = format!("DEVICE_{}", i);
        send_signed_message(&mut channel, &device).await;
    }

    //channel.export_to_file("mypsw", "example/channel_state.json")?;
    Ok((channel_address, announce_id))
}

#[tokio::main]
async fn main(){
    test_channel_create().await.unwrap();
}
