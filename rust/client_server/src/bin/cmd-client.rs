use clap::{Parser, Subcommand};
use client_server::Client;

pub const DEFAULT_PORT: u16 = 6333;

#[derive(Parser, Debug)]
#[command(name = "client")]
#[command(about = "a client framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[clap(name = "hostname", long, default_value = "127.0.0.1")]
    host: String,

    #[clap(long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

#[derive(Subcommand, Debug)]
enum Command {
    Echo {
        msg: String,
    },
    Hello,
}

fn main() {
    let args = Cli::parse();
    let addr = format!("{}:{}", args.host, args.port);
    let mut client = Client::connect(addr).unwrap();

    match args.command {
        Command::Echo { msg } => {
            client.echo(msg).unwrap();
        },
        Command::Hello => {
            client.hello().unwrap();
        },
    }
}
