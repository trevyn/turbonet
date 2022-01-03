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
 public_key: Option<[u8; 96]>,
 proof_of_possession: Option<[u8; 48]>,
 base_url: Option<String>,
}

#[derive(Turbosql, Default)]
struct _Turbonet_Self {
 rowid: Option<i64>,
 ip: Option<u32>,
 port: Option<u16>,
 secret_key: Option<[u8; 32]>,
 public_key: Option<[u8; 96]>,
 proof_of_possession: Option<[u8; 48]>,
 base_url: Option<String>,
}

#[backend]
pub async fn turbonet_heartbeat() -> String {
 "beat!".to_string()
}

/// Start a new Turbonet server. Runs indefinitely.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
 if TURBONET_BOOTSTRAP_IP.is_present() {
  log::info!("TURBONET_BOOTSTRAP_IP is {}", TURBONET_BOOTSTRAP_IP.flag);
  let ip: std::net::Ipv4Addr = TURBONET_BOOTSTRAP_IP.flag.parse()?;
  _Turbonet_Peers {
   ip: Some(ip.into()),
   port: Some(TURBONET_BOOTSTRAP_PORT.flag),
   ..Default::default()
  }
  .insert()?;
 } else {
  log::info!("TURBONET_BOOTSTRAP_IP is NOT PRESENT");
 }

 tokio::spawn(async move {
  tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
  dbg!(remote_turbonet_heartbeat("127.0.0.1:34254").await);
 });

 turbocharger::run_udp_server(TURBONET_LISTEN_PORT.flag).await
}

#[cfg(test)]
mod tests {
 use super::*;

 #[tokio::test]
 async fn test_run() {
  tokio::spawn(async move {
   run().await.unwrap();
  });
  tokio::time::sleep(tokio::time::Duration::from_millis(4000)).await;
 }
}
