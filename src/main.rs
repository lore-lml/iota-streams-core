use std::fs::File;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use iota_streams_lib::{
    utility::{
        iota_utility::create_send_options,
        time_utility::{current_time, TimeUnit}
    }
};
use iota_streams_lib::channel::tangle_channel_reader::ChannelReader;
use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use iota_streams_lib::payload::payload_raw_serializer::{Packet, PacketBuilder};
use iota_streams_lib::user_builders::author_builder::AuthorBuilder;
use iota_streams_lib::user_builders::subscriber_builder::SubscriberBuilder;
use iota_streams_lib::utility::iota_utility::{create_encryption_key, create_encryption_nonce};

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

async fn send_signed_message(channel: &mut ChannelWriter, device_id: &str, key: &[u8; 32], nonce: &[u8;24]){
    println!("Sending message ...");
    let p: Message = get_message(&format!("PUBLIC: {}", device_id));
    let m: Message = get_message(&format!("PRIVATE: {}", device_id));
    let data = PacketBuilder::new()
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

async fn test_channel_create(key: &[u8; 32], nonce: &[u8; 24], channel_psw: &str) -> Result<(String, String)>{
    let send_opt = create_send_options(9, false);

    let author = AuthorBuilder::new()
        .send_options(send_opt)
        .build();

    let mut channel = ChannelWriter::new(author);
    let (channel_address, announce_id) = channel.open().await?;
    println!("Channel: {}:{}", &channel_address, &announce_id);

    for i in 1..=2{
        let device = format!("DEVICE_{}", i);
        send_signed_message(&mut channel, &device, key, nonce).await;
    }

    channel.export_to_file(channel_psw, "example/channel.state")?;
    Ok((channel_address, announce_id))
}

async fn test_restore_channel(key: &[u8; 32], nonce: &[u8; 24], channel_psw: &str) -> Result<()>{
    let send_opt = create_send_options(9, false);

    println!("Restoring Channel ...");
    let (_, mut channel) = ChannelWriter::import_from_file(
        "example/channel.state",
        channel_psw,
        None,
        Some(send_opt)
    )?;
    println!("... Channel Restored");

    let (channel_address, announce_id)= channel.channel_address();
    println!("Channel: {}:{}", &channel_address, announce_id);

    send_signed_message(&mut channel, "DEVICE_3", key, nonce).await;
    Ok(())
}

async fn test_receive_messages(channel_id: String, announce_id: String, key: &[u8; 32], nonce: &[u8; 24]) -> Result<()>{
    let key_nonce = Some((key.clone(), nonce.clone()));

    let sub = SubscriberBuilder::new()
        .send_options(create_send_options(9, false))
        .build();
    let mut reader = ChannelReader::new(sub, &channel_id, &announce_id);
    reader.attach().await?;
    println!("Announce Received");
    let msgs = reader.fetch_parsed_msgs(&key_nonce).await.unwrap() as Vec<(String, Packet)>;
    println!();
    for (id, packet) in msgs {
        println!("Message Found:");
        println!("  Msg Id: {}", id);
        let (p, m): (Message, Message) = packet.parse_data()?;
        println!("  Public: {:?}", p);
        println!("  Private: {:?}\n", m);
    }
    println!("No more messages");
    Ok(())
}

#[tokio::main]
async fn main(){
    let key = create_encryption_key("This is a secret key!!");
    let nonce = create_encryption_nonce("This is a secret nonce");
    let channel_psw = "mypsw";
    let (channel, announce) = test_channel_create(&key, &nonce, channel_psw).await.unwrap();
    test_restore_channel(&key, &nonce, &channel_psw).await.unwrap();
    test_receive_messages(channel, announce, &key, &nonce).await.unwrap();
}
