# Iota Streams-Lib

## Introdution
This Repository provides the base API to open Channels and publish signed data to the Tangle using the `chrysalis-2` branch of iota-streams.<br><br>
This repository has been forked from [iot2tangle/streams-gateway-core](https://docs.iota.org/docs/iota-streams/1.1/overview) <br><br>
This lib allows to :
* Create single branch channels
* Sending signed packets (public data only right now) to the Tangle (and the Tangle only).
* Restoring channels to keep chaining messages to the same one even after the application stops.

To learn more about IOTA-Streams clicke [here](https://docs.iota.org/docs/iota-streams/1.1/overview)

<!--
## Usage
To interact with Library import it as a dependency by adding it to the `Cargo.toml` file:  
`streams_core = { git = "https://github.com/iot2tangle/streams-gateway-core"}`


You can then import the library into your project with:  
`extern crate gateway_core;`
-->

## Author API
To <b>Create a new Author</b> use `AuthorBuilder`:
<br>
`let author = AuthorBuilder::new().node(node_url).send_options(send_opts).encoding(encoding).build().unwrap();`
* `node_url` is the url of a node on a `chrysalis` net.
* `send_opts` is a `SendOptions struct` of the official Iota-streams API.
* `encoding` is the encoding method of data (i.e. `utf-8`).
* Each step of the building process is optional: default values for each field are provided.
<br><br>
To <b>Create a new channel</b> use:  
`Channel::new(author);`
* The author is the one created above. It is suggested to use an author created from the `AuthorBuilder struct` to avoid unexpected behaviours.

To <b>Open the channel</b> and get its address:    
`let (channel_address, announce_id) = channel.open().unwrap();`  
This will open the Channel by generating the channel address and publishing the signature keys.
This address will be needed to read the data from the Tangle
<br>

To <b>Send signed data</b> over the Tangle:  
`write_signed<T>(&mut self, data: T) -> Result<String>`
* The type T needs to have the `serde::Serialize` trait

If the transaction is succesfully sent the id of the attached message will be returned.

To <b>Store and Restore the channel state</b> use `channel.export_to_file(psw, file_path)` and `Channel::imports_from_file(file_path, psw, node_url, send_opts)`:

* `psw` is the password you want to use to encrypt the channel state.
* `file_path` is the path of the file that will be used to store the state.
* `node_url` is an `Option<&str>`: contains the specified url of the nodes as before or `None` for default value.
* `send_opts` is an `Option<SendOptions>`: contains the same struct as before or `None` for default value.

Make sure to use the `export_to_file()` method when you are sure the channel is updated to the last message attached to the tangle or the stored state will be inconsistent.

## Subscriber API
To <b>Create a new Subscriber</b> use `SubscriberBuilder`:
<br>
`let subscriber = SubscriberBuilder::new().node(node_url).send_options(send_opts).encoding(encoding).build().unwrap();`
* `node_url` is the url of a node on a `chrysalis` net.
* `send_opts` is a `SendOptions struct` of the official Iota-streams API.
* `encoding` is the encoding method of data (i.e. `utf-8`).
* Each step of the building process is optional: default values for each field are provided.
<br><br>

To <b>Read from a channel</b> follow these steps:
1. Create the reader:<br>
   `let channel_reader = ChannelReader::new(subscriber, channel_address, announce_id);`
2. Attach the reader to the channel:<br>
   `channel_reader.attach()`
3. Retrieve all msgs on the channel:<br>
   `let msgs = channel_reader.fetch_remaining_msgs();`
4. Loop over them and parse:<br>
   ```
   for m in msgs {
      let address = m.0;
      let data: CustomMessage = Payload::unwrap_data(m.1).unwrap();
      //`CustomMessage` struct must implement serde::{Serialize, Deserialize} traits
      /* collect data */
   }
   ```

## Example
In the `main.rs` file there is a more detailed example on how to send messages and recover channel state.
