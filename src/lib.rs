#![forbid(unsafe_code)]
#![allow(non_camel_case_types)]
#![doc = include_str!("../README.md")]

mod crypto;

pub use crypto::KeyMaterial;
use turbocharger::backend;

gflags::define!(--turbonet_bootstrap_ip: &str);
gflags::define!(--turbonet_bootstrap_port: u16 = 34254);
gflags::define!(--turbonet_listen_port: u16 = 34254);
gflags::define!(--turbonet_heartbeat_interval_seconds: u16 = 3);

use turbosql::Turbosql;

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
}

#[derive(Turbosql, Default)]
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

#[backend]
pub async fn turbonet_heartbeat() -> String {
 "beat!".to_string()
}

/// Spawn a new Turbonet server. Future resolves when the server is ready to accept connections.
pub async fn spawn_server() -> Result<(), Box<dyn std::error::Error>> {
 turbocharger::spawn_udp_server(TURBONET_LISTEN_PORT.flag).await.unwrap();

 if TURBONET_BOOTSTRAP_IP.is_present() {
  log::info!("TURBONET_BOOTSTRAP_IP is {}", TURBONET_BOOTSTRAP_IP.flag);
  let ip: std::net::Ipv4Addr = TURBONET_BOOTSTRAP_IP.flag.parse()?;
  _Turbonet_Peers {
   ip: Some(ip.into()),
   port: Some(TURBONET_BOOTSTRAP_PORT.flag),
   ..Default::default()
  }
  .insert()?;

  tokio::spawn(async move {
   loop {
    dbg!(remote_turbonet_heartbeat(&format!("{}:34254", TURBONET_BOOTSTRAP_IP.flag)).await);
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
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
 async fn test_spawn() {
  spawn_server().await.unwrap();
 }
}
