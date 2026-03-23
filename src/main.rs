mod config;
mod handler;
mod persistence;

use config::Config;
use handler::Handler;
use persistence::{load_approved_messages, save_approved_messages};
use serenity::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    // Parse channels from config
    let config_str = match std::fs::read_to_string("config.json") {
        Ok(config_str) => config_str,
        Err(_) => {
            panic!("Failed to read config.json. Make sure it exists (See config.example.json)");
        }
    };
    let config: Config = serde_json::from_str(&config_str).expect("Failed to parse config.json");

    // Load approved messages from disk if persistence is enabled
    let persistence_file = config
        .persistence_file
        .clone()
        .unwrap_or_else(|| "approved_messages.json".to_string());
    let approved_messages = load_approved_messages(&persistence_file);
    let approved_messages_arc = Arc::new(Mutex::new(approved_messages));

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let approved_messages_for_shutdown = approved_messages_arc.clone();
    let persistence_file_for_shutdown = persistence_file.clone();

    // Set up graceful shutdown handler
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                println!("\nReceived Ctrl+C, saving data and shutting down...");
                let messages = approved_messages_for_shutdown.lock().await;
                save_approved_messages(&persistence_file_for_shutdown, &messages);
                std::process::exit(0);
            }
            Err(err) => {
                eprintln!("Unable to listen for shutdown signal: {}", err);
            }
        }
    });

    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(Handler {
            config,
            approved_messages: approved_messages_arc.clone(),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    // Save on normal shutdown
    let messages = approved_messages_arc.lock().await;
    save_approved_messages(&persistence_file, &messages);
}
