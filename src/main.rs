extern crate thrussh;
extern crate thrussh_keys;
extern crate futures;
extern crate tokio;
extern crate env_logger;
use std::sync::Arc;
use thrussh::*;
use thrussh::server::{Auth, Session};
use thrussh_keys::*;
use futures::Future;
use std::io::Read;
use anyhow::Result;


struct Client {
}

impl client::Handler for Client {
   type FutureUnit = futures::future::Ready<Result<(Self, client::Session), anyhow::Error>>;
   type FutureBool = futures::future::Ready<Result<(Self, bool), anyhow::Error>>;

   fn finished_bool(self, b: bool) -> Self::FutureBool {
       futures::future::ready(Ok((self, b)))
   }
   fn finished(self, session: client::Session) -> Self::FutureUnit {
       futures::future::ready(Ok((self, session)))
   }
   fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
       println!("check_server_key: {:?}", server_public_key);
       self.finished_bool(true)
   }
   fn channel_open_confirmation(self, channel: ChannelId, max_packet_size: u32, window_size: u32, session: client::Session) -> Self::FutureUnit {
       println!("channel_open_confirmation: {:?}", channel);
       self.finished(session)
   }
   fn data(self, channel: ChannelId, data: &[u8], session: client::Session) -> Self::FutureUnit {
       println!("data on channel {:?}: {:?}", channel, std::str::from_utf8(data));
       self.finished(session)
   }
}

#[tokio::main]
async fn main() {
  let config = thrussh::client::Config;
  let config = Arc::new(config);
  let sh = Client{};

  let key = thrussh_keys::key::KeyPair::generate_ed25519().unwrap();
  let mut agent = thrussh_keys::agent::client::AgentClient::connect_env().await.unwrap();
  agent.add_identity(&key, &[]).await.unwrap();
  let mut session = thrussh::client::connect(config, std::env::var("HOST_MONKEY").unwrap(), sh).await.unwrap();
  if session.authenticate_future(std::env::var("USER_MONKEY").unwrap(), key.clone_public_key(), agent).await.unwrap().1 {
    let mut channel = session.channel_open_session().await.unwrap();
    channel.data(b"Hello, world!").await.unwrap();
    if let Some(msg) = channel.wait().await {
        println!("{:?}", msg)
    }
  }
}
