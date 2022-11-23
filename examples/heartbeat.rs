// RUST_LOG=debug cargo run --example heartbeat -- --turbonet_bootstrap_ip=127.0.0.1

gflags::define!(-h, --help = false);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
 tracing_subscriber::fmt::init();
 gflags::parse();

 if HELP.flag {
  gflags::print_help_and_exit(0);
 }

 turbonet::spawn_server("heartbeat").await.unwrap();

 tokio::time::sleep(tokio::time::Duration::from_secs(2000)).await;

 Ok(())
}
