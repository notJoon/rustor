use clap::{Parser, Subcommand};

use crate::model::actor::ActorSystem;

// https://medium.com/@ukpaiugochi0/building-a-cli-from-scratch-with-clapv3-fb9dc5938c82

mod model;
mod test;

#[derive(Parser)]
#[clap(about, version, author)]
struct Value {
    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { n: u32 },
    Recv { msg: String, actor_id: usize },
}

fn cli() {
    let value = Value::parse();

    match value.cmd {
        // `cargo run -- add`
        Commands::Add { n } => {
            println!("Add {} actors", n);
            for _ in 0..n {
                let actor = ActorSystem::new().update_actor_list();
                println!("Actor id: {}", actor);
            }
        }
        Commands::Recv { msg, actor_id } => {
            match msg.to_ascii_uppercase() {
                msg if msg == "STATE" => {
                    let state = ActorSystem::new().get_actor_state(actor_id);
                    println!("{actor_id} state: {state:?}");
                }
                msg if msg == "VALUE" => {
                    let value = ActorSystem::new().get_actor_value(actor_id);
                    println!("{actor_id} value: {value:?}", );
                }
                _ => println!("Invalid message"),
            }
        }
    }
}

fn main() {
    cli();
}
