use tracing::{error, info};
use tracing_subscriber;

use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;

use std::collections::HashMap;
use tokio::sync::Mutex;

mod commands;


#[derive(Debug)]
struct Data {
    command_counter: Mutex<HashMap<String, u64>>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env");

    // Initialize the logger, uses environment variable setting.
    tracing_subscriber::fmt::init();

    let token = std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");

    setup_db().await;
    
    // start stock market

    let framework = poise::Framework::<Data, Error>::builder()
        .options(poise::FrameworkOptions {
            commands: {
                let mut cmds = Vec::new();
                cmds.extend(commands::general::commands());
                cmds.extend(commands::account::commands());
                //cmds.extend(commands::owner::commands());

                cmds
            },

            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                mention_as_prefix: true,
                ..Default::default()
            },

            pre_command: |ctx| {
                Box::pin(async move {
                    let name = ctx.command().name.clone();
                    let mut counter = ctx.data().command_counter.lock().await;
                    *counter.entry(name).or_insert(0) += 1;
                })
            },

            on_error: |error| {
                Box::pin(async move {
                    error!("Command error: {:?}", error);
                })
            },
            ..Default::default()
        })

        .setup(|ctx, ready, _framework| {
            Box::pin(async move {
                info!("{} connected", ready.user.name);

                poise::builtins::register_globally(
                    ctx,
                    &_framework.options().commands,
                )
                .await?;

                Ok(Data {
                    command_counter: Mutex::new(HashMap::new()),
                })
            })
        })
        .build();

    //let intents = GatewayIntents::non_privileged();
    //let intents = GatewayIntents::all();
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILDS
        //| GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Failed to create client");

    if let Err(e) = client.start().await {
        error!("Client error: {:?}", e);
    }
}


/*
#[tokio::main]
async fn main() {
    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }

            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }

        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };
}
*/

async fn setup_db() -> sqlx::SqlitePool {
    use std::str::FromStr;

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let opts = sqlx::sqlite::SqliteConnectOptions::from_str(&db_url)
        .expect("Invalid DATABASE_URL")
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = sqlx::SqlitePool::connect_with(opts)
        .await
        .expect("Couldn't connect to database");

    sqlx::migrate!("./migrations")
        //.run(&db)
        .run(&pool)
        .await
        .expect("Couldn't migrate database.");

    pool
}

