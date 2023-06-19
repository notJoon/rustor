use clap::{Parser, Subcommand};

use crate::model::actor::ActorSystem;

// https://medium.com/@ukpaiugochi0/building-a-cli-from-scratch-with-clapv3-fb9dc5938c82

mod model;
mod test;

#[derive(Parser)]
struct Value {
    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { n: u32 },
}

fn cli() {
    let value = Value::parse();

    match value.cmd {
        Commands::Add { n } => {
            println!("Add {} actors", n);
            for _ in 0..n {
                let actor = ActorSystem::new().add_actor();
                println!("Actor id: {}", actor);
            }
        }
    }
}

fn main() {
    cli();
}
