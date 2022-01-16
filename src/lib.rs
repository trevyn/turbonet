#![forbid(unsafe_code)]
#![allow(non_camel_case_types)]
#![doc = include_str!("../README.md")]

mod crypto;

pub use crypto::KeyMaterial;
use turbocharger::backend;

gflags::define!(--turbonet_bootstrap_ip: &str);
gflags::define!(--turbonet_bootstrap_port: u16 = 34254);
gflags::define!(--turbonet_listen_port: u16 = 34254);
gflags::define!(--turbonet_heartbeat_interval_seconds: u16 = 2);

use turbosql::{select, Turbosql};

#[derive(Turbosql, Default)]
struct _Turbonet_Peers {
 rowid: Option<i64>,
 ip: Option<u32>,
 port: Option<u16>,
 last_seen_ms: Option<i64>,
 crypto_box_public_key: Option<[u8; 32]>,
 bls_public_key: Option<[u8; 96]>,
 bls_proof_of_possession: Option<[u8; 48]>,
 base_url: Option<String>,
 build_id: Option<String>,
}

impl From<SelfResult> for _Turbonet_Peers {
 fn from(item: SelfResult) -> Self {
  _Turbonet_Peers {
   ip: Some(item.ip),
   // port: item.port,
   // last_seen_ms: item.last_seen_ms,
   crypto_box_public_key: Some(item.crypto_box_public_key),
   // bls_public_key: item.bls_public_key,
   // bls_proof_of_possession: item.bls_proof_of_possession,
   base_url: item.base_url,
   build_id: Some(item.build_id),
   ..Default::default()
  }
 }
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
}

impl _Turbonet_Self {
 fn generate() -> Self {
  let mut rng = crypto_box::rand_core::OsRng;
  let crypto_box_secret_key = crypto_box::SecretKey::generate(&mut rng);

  _Turbonet_Self {
   crypto_box_secret_key: Some(crypto_box_secret_key.as_bytes().to_owned()),
   crypto_box_public_key: Some(crypto_box_secret_key.public_key().as_bytes().to_owned()),
   ..Default::default()
  }
 }
}

#[backend]
#[derive(PartialEq)]
struct SelfResult {
 ip: u32,
 // port: u16,
 crypto_box_public_key: [u8; 32],
 // bls_public_key: [u8; 96],
 // bls_proof_of_possession: [u8; 48],
 base_url: Option<String>,
 build_id: String,
}

impl From<_Turbonet_Self> for SelfResult {
 fn from(item: _Turbonet_Self) -> Self {
  #[allow(clippy::or_fun_call)]
  SelfResult {
   ip: 2130706433,
   // port: item.port.unwrap(),
   crypto_box_public_key: item.crypto_box_public_key.unwrap(),
   // bls_public_key: self.bls_public_key,
   // bls_proof_of_possession: self.bls_proof_of_possession,
   base_url: item.base_url,
   build_id: option_env!("BUILD_ID")
    .unwrap_or(format!("DEV {}", option_env!("BUILD_TIME").unwrap_or_default()).as_str())
    .to_owned(),
  }
 }
}

#[backend]
pub async fn turbonet_self() -> SelfResult {
 select!(_Turbonet_Self).unwrap().into()
}

/// Spawn a new Turbonet server. Future resolves when the server is ready to accept connections.
pub async fn spawn_server() -> Result<(), Box<dyn std::error::Error>> {
 let turbonet_self = select!(Option<_Turbonet_Self>)?.unwrap_or_else(|| {
  let turbonet_self = _Turbonet_Self::generate();
  turbonet_self.insert().unwrap();
  turbonet_self
 });

 dbg!(turbonet_self);

 turbocharger::spawn_udp_server(TURBONET_LISTEN_PORT.flag).await.unwrap();

 if TURBONET_BOOTSTRAP_IP.is_present() {
  log::info!("TURBONET_BOOTSTRAP_IP is {}", TURBONET_BOOTSTRAP_IP.flag);
  let ip: std::net::Ipv4Addr = TURBONET_BOOTSTRAP_IP.flag.parse()?;
  let ip: u32 = ip.into();

  tokio::spawn(async move {
   loop {
    let mut peer: _Turbonet_Peers =
     remote_turbonet_self(&format!("{}:34254", TURBONET_BOOTSTRAP_IP.flag)).await.into();
    peer.last_seen_ms =
     Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
      as i64);
    // if we have the peer, update it, otherwise insert it
    if let Some(_Turbonet_Peers { rowid, .. }) =
     select!(Option<_Turbonet_Peers> "WHERE ip = ?", ip).unwrap()
    {
     peer.rowid = rowid;
     peer.update().unwrap();
    } else {
     peer.insert().unwrap();
    }
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
  spawn_server().await.unwrap();
  let peer = remote_turbonet_self("127.0.0.1:34254").await;
  assert_eq!(peer, select!(_Turbonet_Self).unwrap().into());
  dbg!(peer);
 }
}
