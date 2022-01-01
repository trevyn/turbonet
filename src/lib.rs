#![forbid(unsafe_code)]
#![allow(non_camel_case_types)]
#![doc = include_str!("../README.md")]

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
}

/// Start a new Turbonet server. Runs indefinitely.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
 // Insert bootstrap IP if provided.

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

 // Open a UDP listening socket.

 let socket = tokio::net::UdpSocket::bind(format!("0.0.0.0:{}", TURBONET_LISTEN_PORT.flag)).await?;
 log::info!("Listening on: {}", socket.local_addr()?);

 let mut buf = [0; 1500];
 let mut to_send: Option<(usize, std::net::SocketAddr)> = None;

 loop {
  if let Some((size, peer)) = to_send {
   let amt = socket.send_to(&buf[..size], &peer).await?;
   log::info!("Echoed {}/{} bytes to {}", amt, size, peer);
  }

  to_send = Some(socket.recv_from(&mut buf).await?);
 }

 // Send a heartbeat to each peer every `turbonet_heartbeat_interval_seconds` milliseconds.
}
