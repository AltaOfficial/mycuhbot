use once_cell::sync::OnceCell;
use crate::{Data, Error, Context};
use crate::utils::{ConfirmResult, send_hidden_msg, user_choice_from_msg};
use crate::commands::account::EXPIRE_TIME;

static DB_POOL: OnceCell<sqlx::SqlitePool> = OnceCell::new();

pub async fn init_db() {
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
        .run(&pool)
        .await
        .expect("Couldn't migrate database.");

    DB_POOL.set(pool).expect("Database already initialized.");
}

pub fn get_pool() -> &'static sqlx::SqlitePool {
    DB_POOL.get().expect("Database not initialized.")
}

pub async fn init_user_account(ctx: poise::Context<'_, Data, Error>) -> Result<(String, Option<i64>, i64), Error> {
    let user = ctx.author();
    let discord_id = user.id.get() as i64;
    let username = user.name.clone();

    let default_bucks = 0;

    let pool = get_pool();

    let record = sqlx::query!(
        r#"
            INSERT INTO accounts (username, discord_id, bucks)
            VALUES (?, ?, ?) 
            RETURNING username, discord_id, bucks
        "#, 
        username, 
        discord_id, 
        default_bucks
    )
    .fetch_one(pool)
    .await?;

    Ok( (record.username, record.discord_id, record.bucks) )
}


pub async fn ask_user_join_db(ctx: Context<'_>) -> Result<ConfirmResult, Error> {
    let _msg = send_hidden_msg(ctx, "do you want to join the market?" );

    let option = user_choice_from_msg(ctx, ctx.author().id, ctx.id().to_string(), EXPIRE_TIME);

    match option.await {
        ConfirmResult::Yes => { 
            init_user_account(ctx).await?;
            return Ok(ConfirmResult::Yes);
        }
        _ => {}
    };

    Ok(ConfirmResult::No)
}

