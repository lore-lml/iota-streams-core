use streams_core::{
    channel::channel_builder::Channel,
    utility::time_utility::{current_time, TimeUnit}
};
use serde::Deserialize;
use serde::Serialize;
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

fn main() {
    let mut channel = Channel::new("https://nodes.iota.cafe:443".to_string(), 14, false, None);

    let (address, msg_id) = channel.open().unwrap();
    println!("Channel Address: {}", format!("{}:{}", address, msg_id));

    let fr = File::open("example/message.json").unwrap();
    let mut data: Message = serde_json::from_reader(fr).unwrap();
    data.timestamp = current_time(TimeUnit::SECONDS).unwrap();

    match channel.write_signed(data) {
        Ok(_) => {
            println!("Message Attached to the tangle");
        }
        Err(_e) => {
            println!("This isn't working....");
        }
    };
}
