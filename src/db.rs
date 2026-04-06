use once_cell::sync::OnceCell;
use crate::{Data, Error, Context};
use crate::utils::*;

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
    let _msg = send_msg(ctx, "do you want to join the market?", true);

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

// --- standard db functions ---

#[derive(Debug)]
pub struct LeaderboardEntry {
    pub username: String,
    pub bucks:    i64,
}

#[derive(Debug)]
pub struct ProductListing {
    pub product_name:        Option<String>,
    pub product_price:       i64,
    pub product_description: Option<String>,
}

pub async fn fetch_balance(discord_id: i64) -> Result<Option<i64>, Error> {
    let row = sqlx::query!("SELECT bucks FROM accounts WHERE discord_id = ?", discord_id)
        .fetch_optional(get_pool())
        .await?;
    Ok(row.map(|r| r.bucks))
}

/// Returns `(id1_balance, id2_balance)`, or `None` if either user is missing.
pub async fn fetch_two_balances(id1: i64, id2: i64) -> Result<Option<(i64, i64)>, Error> {
    let row = sqlx::query!(
        r#"
            SELECT a1.bucks AS bal1, a2.bucks AS bal2
            FROM accounts AS a1
            INNER JOIN accounts AS a2
                ON a1.discord_id = ? AND a2.discord_id = ?
        "#,
        id1, id2
    )
    .fetch_optional(get_pool())
    .await?;
    Ok(row.map(|r| (r.bal1, r.bal2)))
}

pub async fn update_balance(discord_id: i64, bucks: i64) -> Result<(), Error> {
    sqlx::query!(
        r#"
            UPDATE accounts
            SET bucks = CASE discord_id
                WHEN ? THEN ?
            END
        "#,
        discord_id, bucks
    )
    .execute(get_pool())
    .await?;
    Ok(())
}

pub async fn update_two_balances(id1: i64, bucks1: i64, id2: i64, bucks2: i64) -> Result<(), Error> {
    sqlx::query!(
        r#"
            UPDATE accounts
            SET bucks = CASE discord_id
                WHEN ? THEN ?
                WHEN ? THEN ?
            END
        "#,
        id1, bucks1, id2, bucks2
    )
    .execute(get_pool())
    .await?;
    Ok(())
}

pub async fn fetch_leaderboard() -> Result<Vec<LeaderboardEntry>, Error> {
    let rows = sqlx::query!("SELECT username, bucks FROM accounts ORDER BY bucks DESC")
        .fetch_all(get_pool())
        .await?;
    Ok(rows.into_iter().map(|r| LeaderboardEntry { username: r.username, bucks: r.bucks }).collect())
}

// --- product listing db functions ---

pub async fn create_product(discord_id: i64, name: &str, price: u32, description: &str) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO account_product_listings (discord_id, product_name, product_price, product_description) VALUES (?, ?, ?, ?)",
        discord_id, name, price, description
    )
    .execute(get_pool())
    .await?;
    Ok(())
}

pub async fn remove_product(discord_id: i64, name: &str) -> Result<Option<ProductListing>, Error> {
    let row = sqlx::query!(
        r#"
            DELETE FROM account_product_listings
            WHERE discord_id = ? AND product_name = ?
            RETURNING product_name, product_price, product_description
        "#,
        discord_id, name
    )
    .fetch_optional(get_pool())
    .await?;
    Ok(row.map(|r| ProductListing {
        product_name:        r.product_name,
        product_price:       r.product_price,
        product_description: r.product_description,
    }))
}

pub async fn fetch_product(discord_id: i64) -> Result<Option<ProductListing>, Error> {
    let row = sqlx::query!(
        "SELECT product_name, product_price, product_description FROM account_product_listings WHERE discord_id = ?",
        discord_id
    )
    .fetch_optional(get_pool())
    .await?;
    Ok(row.map(|r| ProductListing {
        product_name:        r.product_name,
        product_price:       r.product_price,
        product_description: r.product_description,
    }))
}



// --- stock market db functions ---

#[derive(Debug)]
pub struct StockListing {
    pub discord_id:       i64,
    pub shares_available: i64,
    pub share_price:      f64,
    pub balance:          f64,
}

#[derive(Debug)]
pub struct StockHolding {
    pub holder_id:  i64,
    pub company_id: i64,
    pub amount:     i64,
}

// --- stock_listings ---

/// Create a new listing row when a user joins the market.
/// `max_stocks` should match `StockMarket::max_stocks` (default 100).
/// Starting balance is $10.00.
pub async fn create_listing(discord_id: i64, max_stocks: i64) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO stock_listings (discord_id, shares_available, share_price, balance)
        VALUES (?, ?, 1.0, 10.0)
        "#,
        discord_id,
        max_stocks,
    )
    .execute(get_pool())
    .await?;

    Ok(())
}

/// Fetch a single user's listing (share count, price, and balance).
pub async fn get_listing(discord_id: i64) -> Result<StockListing, Error> {
    let row = sqlx::query_as!(
        StockListing,
        r#"SELECT discord_id, shares_available, share_price, balance FROM stock_listings WHERE discord_id = ?"#,
        discord_id,
    )
    .fetch_one(get_pool())
    .await?;

    Ok(row)
}

/// Update share count, price, and balance after a trade or market event.
pub async fn update_listing(discord_id: i64, shares_available: i64, share_price: f64, balance: f64) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE stock_listings
        SET shares_available = ?,
            share_price      = ?,
            balance          = ?
        WHERE discord_id = ?
        "#,
        shares_available,
        share_price,
        balance,
        discord_id,
    )
    .execute(get_pool())
    .await?;

    Ok(())
}

/// Remove a user's listing on bankruptcy.
pub async fn delete_listing(discord_id: i64) -> Result<(), Error> {
    sqlx::query!("DELETE FROM stock_listings WHERE discord_id = ?", discord_id)
        .execute(get_pool())
        .await?;

    Ok(())
}

/// Fetch all listings — used to hydrate `StockMarket` on bot startup.
pub async fn get_all_listings() -> Result<Vec<StockListing>, Error> {
    let rows = sqlx::query_as!(
        StockListing,
        "SELECT discord_id, shares_available, share_price, balance FROM stock_listings"
    )
    .fetch_all(get_pool())
    .await?;

    Ok(rows)
}

/// Fetch every holding row — used for market hydration on startup.
pub async fn get_all_holdings() -> Result<Vec<StockHolding>, Error> {
    let rows = sqlx::query_as!(
        StockHolding,
        "SELECT holder_id, company_id, amount FROM stock_holdings"
    )
    .fetch_all(get_pool())
    .await?;

    Ok(rows)
}

/// Rebuild a `StockMarket` from the database.  Called once on bot startup so
/// market state survives restarts.
pub async fn load_market() -> Result<crate::commands::stock_market::StockMarket, Error> {
    use crate::commands::stock_market::{StockMarket, User};
    use std::collections::HashMap;

    let listings = get_all_listings().await?;
    let holdings = get_all_holdings().await?;

    let mut market = StockMarket::new();

    // Build every User entry from the listings table.
    for listing in &listings {
        let id_str = listing.discord_id.to_string();
        // Pre-populate every other company with 0 shares owned so keys always exist.
        let mut company_stock_owned: HashMap<String, u32> = listings
            .iter()
            .filter(|l| l.discord_id != listing.discord_id)
            .map(|l| (l.discord_id.to_string(), 0u32))
            .collect();

        // Overwrite with actual non-zero holdings from the DB.
        for h in holdings.iter().filter(|h| h.holder_id == listing.discord_id) {
            company_stock_owned.insert(h.company_id.to_string(), h.amount as u32);
        }

        market.users.insert(id_str.clone(), User {
            user_id: id_str,
            bal: listing.balance as f32,
            defined_stock_amount_price: (listing.shares_available as u32, listing.share_price as f32),
            company_stock_owned,
        });
    }

    Ok(market)
}

// --- stock_holdings ---

/// Upsert a holding row.  Deletes the row automatically when amount reaches 0
/// to keep the table clean.
pub async fn upsert_holding(holder_id: i64, company_id: i64, new_amount: i64) -> Result<(), Error> {
    if new_amount <= 0 {
        sqlx::query!(
            "DELETE FROM stock_holdings WHERE holder_id = ? AND company_id = ?",
            holder_id,
            company_id,
        )
        .execute(get_pool())
        .await?;
    } else {
        sqlx::query!(
            r#"
            INSERT INTO stock_holdings (holder_id, company_id, amount)
            VALUES (?, ?, ?)
            ON CONFLICT(holder_id, company_id) DO UPDATE SET amount = excluded.amount
            "#,
            holder_id,
            company_id,
            new_amount,
        )
        .execute(get_pool())
        .await?;
    }

    Ok(())
}

/// All holdings for a given holder — used to build `company_stock_owned`.
pub async fn get_holdings_for_holder(holder_id: i64) -> Result<Vec<StockHolding>, Error> {
    let rows = sqlx::query_as!(
        StockHolding,
        "SELECT holder_id, company_id, amount FROM stock_holdings WHERE holder_id = ?",
        holder_id,
    )
    .fetch_all(get_pool())
    .await?;

    Ok(rows)
}

/// All holders of a given company's stock — needed during bankruptcy to return shares.
pub async fn get_holders_of_company(company_id: i64) -> Result<Vec<StockHolding>, Error> {
    let rows = sqlx::query_as!(
        StockHolding,
        "SELECT holder_id, company_id, amount FROM stock_holdings WHERE company_id = ?",
        company_id,
    )
    .fetch_all(get_pool())
    .await?;

    Ok(rows)
}

/// Remove all holding rows that reference a given company (called after bankruptcy).
pub async fn delete_holdings_for_company(company_id: i64) -> Result<(), Error> {
    sqlx::query!("DELETE FROM stock_holdings WHERE company_id = ?", company_id)
        .execute(get_pool())
        .await?;

    Ok(())
}

// --- stock_transactions ---

/// Append an immutable record to the transaction ledger.
pub async fn record_transaction(
    buyer_id:        i64,
    seller_id:       i64,
    amount:          i64,
    price_per_share: f64,
    kind:            &str,   // "buy" | "sell" | "bankruptcy"
) -> Result<(), Error> {
    let total_cost = price_per_share * amount as f64;

    sqlx::query!(
        r#"
        INSERT INTO stock_transactions
            (buyer_id, seller_id, amount, price_per_share, total_cost, kind)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        buyer_id,
        seller_id,
        amount,
        price_per_share,
        total_cost,
        kind,
    )
    .execute(get_pool())
    .await?;

    Ok(())
}

