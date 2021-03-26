use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::iter::FromIterator;

use anyhow::Result;
use iota_streams::{
    app::transport::{
        tangle::{
            client::{Client, SendOptions},
            PAYLOAD_BYTES,
        },
        TransportOptions,
    },
    app_channels::api::tangle::{Address, Author},
};
use serde::Deserialize;
use serde::Serialize;

use streams_core::{
    channel::tangle_channel::Channel,
    utility::time_utility::{current_time, TimeUnit}
};
use streams_core::payload::payload_serializer::empty_bytes;
use streams_core::payload::payload_serializer::json::PayloadBuilder;
use streams_core::payload::payload_serializer::PacketPayload;
use streams_core::users::author_builder::AuthorBuilder;
use streams_core::utility::iota_utility::{create_send_option, random_seed};

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


fn get_message(device_id: String) -> Message{
    let fr = File::open("example/message.json").unwrap();
    let mut data: Message = serde_json::from_reader(fr).unwrap();
    data.timestamp = current_time(TimeUnit::SECONDS).unwrap();
    data.device = device_id;
    data
}

fn save_state(author: &Author<Client>, psw: &str, state_path: &str){
    let author_state = author.export(psw).unwrap();
    let mut fr = OpenOptions::new().write(true).create(true).open(state_path).unwrap();
    fr.write_all(&author_state).unwrap();
}

fn test_channel_create(){
    let data: Message = get_message("DEVICE_1".to_string());

    let send_opt = create_send_option(9, false);
    let node_url = "https://api.lb-0.testnet.chrysalis2.com";

    let mut author = AuthorBuilder::new()
        .send_options(send_opt)
        .node(&node_url)
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

    /*let author_state = author.export("mypsw").unwrap();
    let state_path = "example/author_state.txt";
    let mut fr = File::create(state_path).unwrap();
    fr.write_all(&author_state).unwrap();*/
}

fn test_recover_channel(author_state_path: &str, psw: &str, prev_msgid: &str){
    let mut fr = File::open(author_state_path).unwrap();
    let mut state: Vec<u8> = Vec::<u8>::new();
    fr.read_to_end(&mut state).unwrap();

    let send_opt = create_send_option(9, false);

    let mut client = Client::new_from_url(&"https://api.lb-0.testnet.chrysalis2.com".to_string());
    client.set_send_options(send_opt);

    let mut author = Author::import(&state, psw, client).unwrap();
    let data = get_message("DEVICE_3".to_string());
    let payload = PayloadBuilder::new().public(&data).unwrap().build();

    let link2: Address = author.send_signed_packet(
        &Address::from_str(&author.channel_address().unwrap().to_string(), &prev_msgid).unwrap(),
        &payload.public_data(),
        &empty_bytes(),
    ).unwrap().0;
    let msgid2 = link2.msgid.to_string();
    println!("MSG2 addr: {}", &msgid2);
    save_state(&author, "mypsw", author_state_path);
}

#[tokio::main]
async fn main() {
    //let _appinst = "bc894659b51c8460aa5b65de5a7d08c9a36f20bd1ce91e8e9fa164ac40c57c010000000000000000";
    test_channel_create();
    //test_recover_channel("example/author_state.txt", "mypsw", "45ef9f01d79dd84fd7137c05");
}
