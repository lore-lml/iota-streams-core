use std::fs;
use std::fs::OpenOptions;
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelState{
    author_state: Vec<u8>,
    announcement_id: String,
    last_msg_id: String
}

impl ChannelState {
    pub fn new(author_state: &Vec<u8>, announcement_id: &str, last_msg_id: &str) -> ChannelState{
        ChannelState{
            author_state: author_state.clone(),
            announcement_id: announcement_id.to_string(),
            last_msg_id: last_msg_id.to_string()
        }
    }

    pub fn from_file(file_path: &str) -> Result<ChannelState>{
        let fr = OpenOptions::new().read(true).open(file_path).unwrap();
        let channel_state: ChannelState = serde_json::from_reader(fr).unwrap();
        Ok(channel_state)
    }
}

impl ChannelState{
    pub fn write_to_file(&self, file_path: &str){
        match fs::remove_file(file_path){_ => ()};
        let fr = OpenOptions::new().write(true).create(true).open(file_path).unwrap();
        match serde_json::to_writer(fr, &self){
            Ok(_) => (),
            Err(_) => eprintln!("Error during IO operations")
        }
    }

    pub fn author_state(&self) -> Vec<u8> {
        self.author_state.clone()
    }
    pub fn announcement_id(&self) -> String {
        self.announcement_id.clone()
    }
    pub fn last_msg_id(&self) -> String {
        self.last_msg_id.clone()
    }
}
