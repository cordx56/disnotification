mod db;
mod discord;

use serenity::{prelude::GatewayIntents, Client};
use std::env;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not set");

    let db_pool = db::connect().await.expect("DB connect error");
    let mut client = Client::builder(token, GatewayIntents::GUILD_VOICE_STATES)
        .event_handler(discord::DisNotHandler { db_pool: db_pool })
        .await
        .expect("Client creating error");

    if let Err(e) = client.start().await {
        println!("Client error: {:?}", e);
    }
}
