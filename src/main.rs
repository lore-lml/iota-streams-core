use serde::{Deserialize, Serialize};

use iota_streams_lib::{
    utility::{
        iota_utility::create_send_options,
        time_utility::{current_time, TimeUnit}
    }
};

use iota_streams_lib::utility::iota_utility::create_link;
use iota_streams_lib::user_builders::author_builder::AuthorBuilder;
use iota_streams_lib::user_builders::subscriber_builder::SubscriberBuilder;
use iota_streams_lib::channel::tangle_channel_writer::ChannelWriter;
use iota_streams_lib::channel::tangle_channel_reader::ChannelReader;
use std::fs::File;

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

#[allow(dead_code)]
fn get_message(device_id: &str) -> Message{
    let fr = File::open("example/message.json").unwrap();
    let mut data: Message = serde_json::from_reader(fr).unwrap();
    data.timestamp = current_time(TimeUnit::SECONDS).unwrap();
    data.device = device_id.to_string();
    data
}
/*
fn send_signed_message(channel: &mut ChannelWriter, device_id: &str){
    println!("Sending message ...");
    let data: Message = get_message(device_id);
    let msg_id = channel.send_signed_packet(&data, false).unwrap();
    println!("... Message sent:");
    println!("id -> {}", msg_id);
    println!("data -> {:?}\n\n", &data);
}*/

#[allow(dead_code)]
async fn test_multibranch(){
    let author = AuthorBuilder::new().send_options(create_send_options(9, false)).build();
    let subscriber = SubscriberBuilder::new().send_options(create_send_options(9, false)).build();
    let non_subscriber = SubscriberBuilder::new().send_options(create_send_options(9, false)).build();

    let mut writer = ChannelWriter::new(author);
    let (channel, announce) = writer.open().await.unwrap();
    println!("Channel: {}:{}", channel, announce);

    let mut sub_reader = ChannelReader::new(subscriber, &channel, &announce);
    println!("Sub Announce received: {}", sub_reader.attach().await.is_ok());
    let mut nonsub_reader = ChannelReader::new(non_subscriber, &channel, &announce);
    println!("Non_Sub Announce received: {}", nonsub_reader.attach().await.is_ok());

    let sub_id = sub_reader.send_subscription().await.unwrap();
    println!("Subscription Received: {}:{}", sub_id, writer.get_single_sub(sub_id.clone()).await.unwrap());

    let keyload = writer.start_private_communications().await.unwrap();
    println!("Keyload sent: {}", keyload);

    let masked_payload = "PRIVATE 1".as_bytes().to_vec();
    writer.send_signed_raw_data(masked_payload, true).await.unwrap();
    let masked_payload = "PRIVATE 2".as_bytes().to_vec();
    writer.send_signed_raw_data(masked_payload, true).await.unwrap();
    println!("Masked Data Sent");

    let public_payload = "PUBLIC 1".as_bytes().to_vec();
    let pub1 = writer.send_signed_raw_data(public_payload, false).await.unwrap();
    let public_payload = "PUBLIC 2".as_bytes().to_vec();
    let pub2 = writer.send_signed_raw_data(public_payload, false).await.unwrap();
    println!("Public Data Sent: \n{}\n{}", pub1, pub2);

    let masked_payload = "PRIVATE 3".as_bytes().to_vec();
    writer.send_signed_raw_data(masked_payload, true).await.unwrap();

    let public_payload = "PUBLIC 3".as_bytes().to_vec();
    let pub3 = writer.send_signed_raw_data(public_payload, false).await.unwrap();
    println!("Public Data Sent: \n{}", pub3);


    sub_reader.get_all_msgs().await;
    nonsub_reader.get_all_msgs().await;
}

async fn test_non_sub_reader(){
    let channel = "e202390c23fc44b13dbebc24a1806a76fdf380915da51589d8427ac23cd029280000000000000000";
    let announce = "e0347f47ac06b68351dbfac1";
    let msg1 = "cf316b63b4f30c10178f1a24";
    let msg2 = "392fb978481812b008cebc8b";
    let msg3 = "f29fd09edd60b4875e971b01";
    let mut sub = SubscriberBuilder::new().send_options(create_send_options(9, false)).build();
    let res = sub.receive_announcement(&create_link(channel, announce).unwrap()).await.is_ok();
    println!("Announce: {}", res);

    let (_, p1, m1) = sub.receive_signed_packet(&create_link(channel, msg1).unwrap()).await.unwrap();
    let (_, p2, m2) = sub.receive_signed_packet(&create_link(channel, msg2).unwrap()).await.unwrap();
    let (_, p3, m3) = sub.receive_signed_packet(&create_link(channel, msg3).unwrap()).await.unwrap();

    println!("{}\n{}", String::from_utf8(p1.0).unwrap(), String::from_utf8(m1.0).unwrap());
    println!("{}\n{}", String::from_utf8(p2.0).unwrap(), String::from_utf8(m2.0).unwrap());
    println!("{}\n{}", String::from_utf8(p3.0).unwrap(), String::from_utf8(m3.0).unwrap());
}

#[tokio::main]
async fn main(){
    test_multibranch().await;
    test_non_sub_reader().await;
}
