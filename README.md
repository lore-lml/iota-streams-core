# Iota Streams-Lib

## Introdution
This Repository provides the base API to open Channels, publish and receive signed data to the Tangle using the `chrysalis-2` branch of iota-streams.
<br><br>
NOTE: `async` version of IOTA-streams API has been used.

This lib allows to :
* Create single branch channels
* Send signed packets public data to the Tangle (and the Tangle only).
* Each packet can be split in two parts:
   * Public part that can be read from anyone who have access to channel.
   * Masked part optionally encrypted with the [Xchacha20-poly1305 authenticated encryption algorithm](https://tools.ietf.org/html/draft-arciszewski-xchacha-03)
* Restoring channels to keep chaining messages to an already existing channel, even after the application stops.
* Receive signed packets from a channel.

To learn more about IOTA-Streams click [here](https://docs.iota.org/docs/iota-streams/1.1/overview)


## Usage
First, make sure to use the latest rust version using the command `rustup update`.

To interact with Library implementations you can add the dependency in `Cargo.toml` file:
```
[dependencies]
iota_streams_lib = { git = "https://github.com/lore-lml/iota-streams-lib.git"}
```

You can then import the library into your project with:  
`use iota_streams_lib::*;`

## Author API
### To Create a new Author use:

```
let author = AuthorBuilder::new()
            .node(node_url)
            .send_options(send_opts)
            .encoding(encoding)
            .build();
```

* `node_url` is the url of a node on a `chrysalis` net.
* `send_opts` is a `SendOptions struct` of the official Iota-streams API.
* `encoding` is the encoding method of data (i.e. `utf-8`).
* Each step of the building process is optional: default values for each field are provided.

### To Create and Write into a new channel use:  
```let mut channel = ChannelWriter::new(author);```
* The author is the one created above. It is suggested to use an author created from the `AuthorBuilder struct` to avoid unexpected behaviours.

### To Open the channel and get its address:    
```let (channel_address, announce_id) = channel.open().unwrap();```  
This will open the Channel by generating the channel address and publishing the signature keys.
This address will be needed to read the data from the Tangle
<br>

### To Send signed raw data over the Tangle:  
`async fn send_signed_raw_data(&mut self, p_data: Vec<u8>, m_data: Vec<u8>, key_nonce: Option<([u8;32], [u8;24])>) -> Result<String>`<br>
* `p_data:` it's a bytes vector containing the public part of the packet.
* `m_data:` it's a bytes vector containing the masked part of the packet.
* `key_nonce:` it's an optional tuple of fixed byte array containing the `encryption key` and `nonce`.
This option enables the encryption with the given key and nonce for the masked part of the packet.

If the transaction is succesfully sent the id of the attached message will be returned.

### To Send signed packet, built from custom struct, over the Tangle:
`async fn send_signed_packet<T>(&mut self, packet: &StreamsPacket<T>) -> Result<String>`
* The type T needs to have the `StreamsPacketSerializer` trait.
* For an easier use you can build a valid `StreamsPacket<T>` using a `Packet` struct.<br>

If the transaction is successfully sent the id of the attached message will be returned.
  
### To Create a valid packet use:
```
   let packet = PacketBuilder::new()
   .public(&p_data).unwrap()
   .masked(&m_data).unwrap()
   .build()
   ```


### To Store and Restore the channel state use:
```
let channel = ChannelWriter::new(author);
/* ********** Do stuff ********** */

channel.export_to_file(psw, file_path);
/* ********** applications stops ********** */

let channel = ChannelWriter::imports_from_file(file_path, psw, node_url, send_opts);
/* ********** Do stuff as the application never stops ********** */
```

* `psw` is the password you want to use to encrypt the channel state.
* `file_path` is the path of the file that will be used to store the state.
* `node_url` is an `Option<&str>`: contains the specified url of the nodes as before or `None` for default value.
* `send_opts` is an `Option<SendOptions>`: contains the same struct as before or `None` for default value.

NOTE: Make sure to use the `export_to_file()` method when you are sure the channel is updated to the last message attached to the tangle or the stored state will be inconsistent.

## Subscriber API
### To Create a new Subscriber use:
```
let subscriber = SubscriberBuilder::new()
                  .node(node_url)
                  .send_options(send_opts)
                  .encoding(encoding)
                  .build();
```
* `node_url` is the url of a node on a `chrysalis` net.
* `send_opts` is a `SendOptions struct` of the official Iota-streams API.
* `encoding` is the encoding method of data (i.e. `utf-8`).
* Each step of the building process is optional: default values for each field are provided.

### To Receive packets from a channel follow these steps:
1. Create the reader:<br>
   ```let channel_reader = ChannelReader::new(subscriber, channel_address, announce_id);```
2. Attach the reader to the channel:<br>
   ```channel_reader.attach()```
3. Retrieve all msgs on the channel:<br>
   ```let msgs = channel_reader.fetch_raw_msgs(key_nonce);```<br>
   or<br>
   ```let msgs = channel_reader.fetch_parsed_msgs(key_nonce);```
4. Loop over them and parse.


## Utility API
* `fn random_seed() -> String`: 
  creates a random seed of 81 chars.
* `fn create_send_options(min_weight_magnitude: u8, local_pow: bool) -> SendOptions`:
  creates a the `SendOptions` struct needed for author and subscriber with the specified `minimum weighted magnitude`.
* `fn hash_string(string: &str) -> String`:
  it creates the digest of a string using `blake2b`.
* `fn create_link(appinst: &str, msg_id: &str) -> Result<Address>`:
  it creates the official IOTA-streams `Address` struct with the specified `appinst` and `msg_id`.
* `fn create_encryption_key(string_key: &str) -> [u8; 32]`:
  it creates the corresponding key bytes array needed for the encryption and decryption of the masked part of the packet,
  starting from a secret string.
* `fn create_encryption_nonce(string_nonce: &str) -> [u8;24]`:
  it creates the corresponding nonce bytes array needed for the encryption and decryption of the masked part of the packet,
  starting from a secret string.

## Example
In the `example` folder there is a more detailed example on how to send and receive packets to/from the tangle,
and recover channel state.

Use `cargo run --package example` to run the example application.

## Notes
In this lib an external approach for the encryption of the masked payload has been used.<br>
IOTA-streams framework is currently in the alpha stage, so even if it should provide an internal protocol for key distribution and 
a mechanism to send both public and masked data in the same packet, right now it has some limitations, and it's not easy to use.
Knowing this, you should use your own key distribution protocol in order to make subscribers able to 
decrypt the encrypted data of a packet.
