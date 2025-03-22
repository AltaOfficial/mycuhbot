use serenity::{all::{ClientBuilder, GatewayIntents}, Client, Result, client::EventHandler, async_trait};
use std::env;
use dotenv::dotenv;
use serenity::prelude::*;
use serenity::model::gateway::Ready;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is online!", ready.user.name)
    }
}

pub async fn get_client() -> Result<Client> {
    dotenv().ok();
    let token: String = env::var("DISCORD_TOKEN").expect("Expected a discord token");

    Ok(Client::builder(token, GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MEMBERS).event_handler(Handler).await.expect("Error creating client"))
}