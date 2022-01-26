#![forbid(unsafe_code)]
#![allow(non_camel_case_types)]
#![doc = include_str!("../README.md")]

mod crypto;

use serde_with::serde_as;
use turbocharger::backend;
use turbosql::{select, Turbosql};

gflags::define!(--turbonet_bootstrap_ip: &str);
gflags::define!(--turbonet_bootstrap_port: u16 = 34254);
gflags::define!(--turbonet_listen_port: u16 = 34254);
gflags::define!(--turbonet_heartbeat_interval_seconds: u16 = 2);

fn now_millis() -> i64 {
 std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
}

// SELECT rowid,ip,last_request_ms,last_response_ms,build_id,substr(hex(crypto_box_public_key), 1, 8) AS cb_public FROM _turbonet_peer;
#[derive(Turbosql, Default, Debug)]
struct _Turbonet_Peer {
 rowid: Option<i64>,
 ip: Option<u32>,
 port: Option<u16>,
 last_request_ms: Option<i64>,
 last_response_ms: Option<i64>,
 crypto_box_public_key: Option<[u8; 32]>,
 bls_public_key: Option<[u8; 96]>,
 bls_proof_of_possession: Option<[u8; 48]>,
 base_url: Option<String>,
 build_id: Option<String>,
}

#[derive(Turbosql, Default, Debug)]
struct _Turbonet_Self {
 rowid: Option<i64>,
 ip: Option<u32>,
 port: Option<u16>,
 crypto_box_secret_key: Option<[u8; 32]>,
 crypto_box_public_key: Option<[u8; 32]>,
 bls_secret_key: Option<[u8; 32]>,
 bls_public_key: Option<[u8; 96]>,
 bls_proof_of_possession: Option<[u8; 48]>,
 base_url: Option<String>,
 build_id: Option<String>,
}

impl _Turbonet_Self {
 fn generate_keys() -> Self {
  let mut rng = crypto_box::rand_core::OsRng;
  let crypto_box_secret_key = crypto_box::SecretKey::generate(&mut rng);

  let bls = crypto::KeyMaterial::generate_new();

  _Turbonet_Self {
   crypto_box_secret_key: Some(crypto_box_secret_key.as_bytes().to_owned()),
   crypto_box_public_key: Some(crypto_box_secret_key.public_key().as_bytes().to_owned()),
   bls_secret_key: Some(bls.secret_key),
   bls_public_key: Some(bls.public_key),
   bls_proof_of_possession: Some(bls.proof_of_possession),
   ..Default::default()
  }
 }
}

impl From<_Turbonet_Self> for _Turbonet_SelfResponse {
 fn from(item: _Turbonet_Self) -> _Turbonet_SelfResponse {
  _Turbonet_SelfResponse {
   crypto_box_public_key: item.crypto_box_public_key.unwrap(),
   bls_public_key: item.bls_public_key.unwrap(),
   bls_proof_of_possession: item.bls_proof_of_possession.unwrap(),
   base_url: item.base_url,
   build_id: item.build_id.unwrap(),
  }
 }
}

#[serde_as]
#[backend]
#[derive(PartialEq, Debug)]
struct _Turbonet_SelfResponse {
 crypto_box_public_key: [u8; 32],
 #[serde_as(as = "[_; 96]")]
 bls_public_key: [u8; 96],
 #[serde_as(as = "[_; 48]")]
 bls_proof_of_possession: [u8; 48],
 base_url: Option<String>,
 build_id: String,
}

#[backend]
pub async fn _turbonet_self() -> _Turbonet_SelfResponse {
 select!(_Turbonet_Self).unwrap().into()
}

impl _Turbonet_Peer {
 async fn heartbeat(self) {
  let mut peer = if self.rowid.is_some() {
   self
  } else if let Some(peer) =
   select!(Option<_Turbonet_Peer> "WHERE ip = ?", self.ip.unwrap()).unwrap()
  {
   peer
  } else {
   _Turbonet_Peer { rowid: Some(self.insert().unwrap()), ..self }
  };

  peer.last_request_ms = Some(now_millis());
  peer.update().unwrap();

  let response = remote__turbonet_self(&format!(
   "{}:{}",
   std::net::Ipv4Addr::from(peer.ip.unwrap()),
   peer.port.unwrap()
  ))
  .await;

  peer.crypto_box_public_key = Some(response.crypto_box_public_key);
  peer.bls_public_key = Some(response.bls_public_key);
  peer.bls_proof_of_possession = Some(response.bls_proof_of_possession);
  peer.build_id = Some(response.build_id);
  peer.base_url = response.base_url;
  peer.last_response_ms = Some(now_millis());
  peer.update().unwrap();
 }
}

/// Spawn a new Turbonet server. Future resolves when the server is ready to accept connections.
pub async fn spawn_server(build_id: &str) -> Result<(), Box<dyn std::error::Error>> {
 let mut turbonet_self = select!(Option<_Turbonet_Self>)?.unwrap_or_else(|| {
  let turbonet_self = _Turbonet_Self::generate_keys();
  turbonet_self.insert().unwrap();
  turbonet_self
 });

 turbonet_self.rowid = Some(1);
 turbonet_self.build_id = Some(build_id.to_owned());
 turbonet_self.update()?;

 // dbg!(turbonet_self);

 turbocharger::spawn_udp_server(TURBONET_LISTEN_PORT.flag).await.unwrap();

 if TURBONET_BOOTSTRAP_IP.is_present() {
  log::info!("TURBONET_BOOTSTRAP_IP is {}", TURBONET_BOOTSTRAP_IP.flag);
  let ip: std::net::Ipv4Addr = TURBONET_BOOTSTRAP_IP.flag.parse()?;
  let ip: u32 = ip.into();

  tokio::spawn(async move {
   loop {
    _Turbonet_Peer { ip: Some(ip), port: Some(TURBONET_BOOTSTRAP_PORT.flag), ..Default::default() }
     .heartbeat()
     .await;
    tokio::time::sleep(tokio::time::Duration::from_secs(
     TURBONET_HEARTBEAT_INTERVAL_SECONDS.flag.into(),
    ))
    .await;
   }
  });
 } else {
  log::info!("TURBONET_BOOTSTRAP_IP is NOT PRESENT");
 }

 Ok(())
}

#[cfg(test)]
mod tests {
 use super::*;

 #[tokio::test]
 async fn test_server() {
  spawn_server("test").await.unwrap();
  let peer = remote__turbonet_self("127.0.0.1:34254").await;
  assert_eq!(peer, select!(_Turbonet_Self).unwrap().into());
  dbg!(peer);
 }
}
