use iota_streams::app::transport::tangle::{
    PAYLOAD_BYTES,
    client::{SendOptions, Client as StreamsClient}
};
use crate::utility::iota_utility::random_seed;
use iota_streams::app_channels::api::tangle::Subscriber;
use iota_streams::app::transport::TransportOptions;

pub struct SubscriberBuilder{
    seed: String,
    node_url: String,
    encoding: String,
    send_options: SendOptions
}

impl SubscriberBuilder{
    pub fn new() -> SubscriberBuilder{
        let mut send_opts = SendOptions::default();
        send_opts.local_pow = false;

        SubscriberBuilder{
            seed: random_seed(),
            node_url: "https://api.lb-0.testnet.chrysalis2.com".to_string(),
            encoding: "utf-8".to_string(),
            send_options: send_opts
        }
    }

    /*pub fn build_from_state(author_state: &[u8],
                            psw: &str,
                            node_url: Option<&str>,
                            send_option: Option<SendOptions>){}*/
}

impl SubscriberBuilder{
    pub fn seed(mut self, seed: &str) -> Self{
        self.seed = seed.to_string();
        self
    }

    pub fn node(mut self, node_url: &str) -> Self{
        self.node_url = node_url.to_string();
        self
    }

    pub fn encoding(mut self, encoding: &str) -> Self{
        self.encoding = encoding.to_string();
        self
    }

    pub fn send_options(mut self, send_options: SendOptions) -> Self{
        self.send_options = send_options;
        self
    }

    pub fn build(self) -> Subscriber<StreamsClient>{
        let mut client = StreamsClient::new_from_url(&self.node_url);
        client.set_send_options(self.send_options);
        Subscriber::new(&self.seed,
                        &self.encoding,
                        PAYLOAD_BYTES,
                        client)
    }
}
