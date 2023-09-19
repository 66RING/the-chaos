use clap::{Parser, Subcommand};
use client_server::Server;

pub const DEFAULT_PORT: u16 = 6333;

#[derive(Parser, Debug)]
#[command(name = "server")]
#[command(about = "a client framework", long_about = None)]
struct Cli {
    #[clap(name = "hostname", long, default_value = "127.0.0.1")]
    host: String,

    #[clap(long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

fn main() {
    let args = Cli::parse();
    let addr = format!("{}:{}", args.host, args.port);
    let server = Server::new();

    server.run(addr).unwrap();
}

