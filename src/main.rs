use std::fs::File;

use anyhow::{Result, Error};
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
use iota_streams::ddml::types::Bytes;
use iota_streams_lib::payload::payload_serializer::empty_bytes;
use iota_streams::core_edsig::signature::ed25519::PublicKey;
use iota_streams::app::message::HasLink;
use std::time::Duration;
use iota_streams_lib::utility::iota_utility::create_link;
use iota_streams::app_channels::api::tangle::{Address, MessageContent};

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
        .build();

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
    let subscriber = SubscriberBuilder::new().send_options(send_opts).build();

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

fn test_masked_message() -> Result<Address>{
    let mut author = AuthorBuilder::new()
        .node("https://api.hornet-0.testnet.chrysalis2.com")
        .send_options(create_send_options(9, false))
        .build();

    /* ******************** ANNOUNCE ************************** */
    let announce = author.send_announce()?;
    let (channel_address, announce_id) = (announce.appinst.to_string(), announce.msgid.to_string());
    println!("Announce Channel: {}:{}\n", &channel_address, &announce_id);

    let mut sub1 = SubscriberBuilder::new()
        .node("https://api.hornet-1.testnet.chrysalis2.com")
        .send_options(create_send_options(9, false))
        .build();

    sub1.receive_announcement(&announce)?;
    println!("Sub1 Received announce: {}:{}", &channel_address, &announce_id);

    let mut sub2 = SubscriberBuilder::new()
        .node("https://api.hornet-2.testnet.chrysalis2.com")
        .send_options(create_send_options(9, false))
        .build();

    sub2.receive_announcement(&announce)?;
    println!("Sub2 Received announce: {}:{}\n", &channel_address, &announce_id);

    /* ******************** SUBSCRIBE ************************** */

    let link_sub1 = sub1.send_subscribe(&announce)?;
    println!("Sub1 Subscribe sent: {}", link_sub1.msgid.to_string());
    let link_sub2 = sub2.send_subscribe(&announce)?;
    println!("Sub2 Subscribe sent: {}\n", link_sub2.msgid.to_string());
    author.receive_subscribe(&link_sub1)?;
    author.receive_subscribe(&link_sub2)?;
    println!("Subscriptions received:\n{}\n{}\n", link_sub1.msgid.to_string(), link_sub2.msgid.to_string());

    /* ******************** Keyload for everyone ************************** */

    std::thread::sleep(Duration::from_secs(1));
    let keyload_for_all = author.send_keyload_for_everyone(&announce)?.0;
    println!("Keyload sent with id: {}", keyload_for_all.msgid.to_string());

    sub1.receive_keyload(&keyload_for_all)?;
    println!("Sub1 received keyload with id: {}", keyload_for_all.msgid.to_string());

    sub2.receive_keyload(&keyload_for_all)?;
    println!("Sub2 received keyload with id: {}\n", keyload_for_all.msgid.to_string());


    /* ******************** Masked messages ************************** */

    let data_string = "hello world";
    let data = Bytes(data_string.as_bytes().to_vec());

    let m1 = author.send_signed_packet(&keyload_for_all, &empty_bytes(), &data)?.0;
    let (_, _, m_data) = sub1.receive_signed_packet(&m1)?;
    let m_data = String::from_utf8(m_data.0)?;
    println!("Sub1:\nfound -> {}\nexpected -> {}", m_data, data_string);


    let (_, _, m_data) = sub2.receive_signed_packet(&m1)?;
    let m_data = String::from_utf8(m_data.0)?;
    println!("Sub2:\nfound -> {}\nexpected -> {}", m_data, data_string);



    /* ******************** Keyload for sub1 only ************************** */

    /*let keyload_sub1 = author.send_keyload_for_everyone(&link_sub1)?.0;
    println!("Keyload for Sub1 sent with id: {}\n", keyload_sub1.msgid.to_string());

    sub1.receive_keyload(&keyload_sub1)?;
    println!("Sub1 received HIS KEYLOAD for Sub1 with id: {}\n", keyload_sub1.msgid.to_string());

    match sub2.receive_keyload(&keyload_sub1){
        Ok(_) => println!("Sub2 received keyload for Sub1: SOMETHING WENT WRONG"),
        Err(_) => println!("Sub2 is not able to receive keyload for Sub1: EVERYTHING's OK"),
    }
    std::thread::sleep(Duration::from_secs(1));


    /* ******************** Masked messages ************************** */

    let data_string = "JUST FOR SUB1";
    let data = Bytes(data_string.as_bytes().to_vec());

    let m1 = author.send_signed_packet(&keyload_sub1, &empty_bytes(), &data)?.0;
    let (_, _, m_data) = sub1.receive_signed_packet(&m1)?;
    let m_data = String::from_utf8(m_data.0)?;
    println!("Sub1:\nfound -> {}\nexpected -> {}", m_data, data_string);
    std::thread::sleep(Duration::from_secs(1));

    let m_data = match sub2.receive_signed_packet(&m1){
        Ok(res) => res.2,
        Err(_) => Bytes("Unknown".as_bytes().to_vec())
    };
    let m_data = String::from_utf8(m_data.0)?;
    println!("Sub2:\nfound -> {}\nexpected -> Unknown", m_data);
    std::thread::sleep(Duration::from_secs(1));*/


    Ok(announce)
}

fn public_sub(announce: &Address) -> Result<()>{

    /* ******************** public subscriber ************************** */

    let mut p_sub = SubscriberBuilder::new()
        .send_options(create_send_options(9, false))
        .build();

    p_sub.receive_announcement(announce)?;
    println!("PSUB found announcement\n");
    let mut msgs: Vec<String> = Vec::new();
    let mut has_next = true;
    while has_next{

        let mut m: Vec<String> = p_sub.fetch_next_msgs().iter()
            .map(|msg|  {
                let link = msg.link.rel().to_string();
                match &msg.body{
                    MessageContent::Announce => println!("Announce"),
                    MessageContent::Keyload => println!("Keyload"),
                    MessageContent::SignedPacket {public_payload, masked_payload, ..} => {
                        println!("Found signed packet msg with id: {}", &link);
                        let p_data = String::from_utf8(public_payload.0.clone()).unwrap();
                        let m_data = String::from_utf8(masked_payload.0.clone()).unwrap();
                        println!("Public data: {}", p_data);
                        println!("Masked data: {}\n", m_data);
                    }
                    MessageContent::Subscribe => println!("Subscribe"),
                    _ => println!("ALTRO"),
                };
                link
            })
            .collect();

        if m.len() > 0{
            msgs.append(&mut m);
        }else{
            has_next = false;
        }
    }

    println!("Msgs found {}, expected 0", msgs.len());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    /*match start_test() {
        Ok(_) => println!("\n******************** Everything has worked ********************"),
        Err(_) => println!("\n******************** Something went wrong ********************")
    }*/
    //let announce = test_masked_message()?;
    let c = "010ae1e97a2fc41efca894757ef274792df7bcf1095d811d0b0b56c7de67f10f0000000000000000";
    let a = "e010d8662840983577c8f73b";
    let announce = create_link(c,a).unwrap();
    public_sub(&announce);
    Ok(())
}
