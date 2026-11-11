use core::f32;
use std::collections::HashMap;
use std::fmt;
use rand::RngExt;

#[derive(Debug)]
pub struct User {
    pub user_id: String, // may not need, already exist in 'users' hashmap
    pub bal: f32,
    pub defined_stock_amount_price: (u32, f32), // (amount, price) 
    pub company_stock_owned: HashMap<String, u32>, // <ID, amount> Other companies they own stock in
}

impl fmt::Display for User {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "User ID: {}\nBalance: ${}\nDefined Stock: {} at ${}\nOwned Stocks: {:?}",
            self.user_id,
            self.bal,
            self.defined_stock_amount_price.0,
            self.defined_stock_amount_price.1,
            self.company_stock_owned
        )
    }
}

#[derive(Debug)]
pub struct StockMarket {
    pub users: HashMap<String, User>, // Mapping of user_id to User object
    pub max_stocks: u32,
}


impl StockMarket {
    pub fn new() -> StockMarket {
        StockMarket {
            users: HashMap::new(),
            max_stocks: 100, // TODO possibly get from database
        }
    }

    //#[cfg(test)]
    pub fn get_users(&self) -> &HashMap<String, User> {
        &self.users
    }

    //#[cfg(test)]
    pub fn get_max_stocks(&self) -> u32 {
        self.max_stocks
    }

    // Add a user to the market with a defined stock
    pub fn add_user(&mut self, id: &str) {
        // adds new user to all other users list
        let id_str = id.to_string();
        for (_, user) in self.users.iter_mut() {
            user.company_stock_owned.insert(id_str.clone(), 0);
        }

        let mut new_user = User {
            user_id: id_str.clone(),
            bal: 10.0,
            defined_stock_amount_price: (self.max_stocks, 1.0), // Default stock: 100 units at price 10.0
            company_stock_owned: HashMap::new(), // No other stock owned initially
        };
        // adds all other users to new user stock list
        for company_id in self.users.keys() {
            new_user.company_stock_owned.insert(company_id.clone(), 0);
        }

        // add new user to whole list
        self.users.insert(id_str, new_user);
    }

    pub fn get_user_info(&self, id: &str) -> &User {
        self.users.get(id).unwrap()
    }

    pub fn buy_stock(&mut self, buyer_id: &str, seller_id: &str, amount: u32) {
        self.trade_stock(buyer_id, seller_id, amount, true);
    }

    pub fn sell_stock(&mut self, seller_id: &str, company_id: &str, amount: u32) {
        self.trade_stock(company_id, seller_id, amount, false); // Selling is just the reverse of buying in logic
    }

    fn trade_stock(&mut self, buyer_id: &str, seller_id: &str, amount: u32, is_buying: bool) {
        // names buyer and seller make it easier to read, logic changes based on higher order
        // functions
        if let (Some(mut buyer), Some(mut seller)) = (
            self.users.remove(buyer_id), 
            self.users.remove(seller_id)
        ){
            // Ensure buyer has enough balance and seller has enough stock to sell
            // buy shares of company
            if is_buying {
                let cost = seller.defined_stock_amount_price.1 * amount as f32;
                if buyer.bal >= cost && seller.defined_stock_amount_price.0 >= amount {
                    buyer.bal -= cost;
                    seller.bal += cost;

                    // Update stock ownership
                    buyer.company_stock_owned.entry(seller_id.to_string()).and_modify(|stocks| *stocks += amount);
                    seller.company_stock_owned.entry(seller_id.to_string()).and_modify(|stocks| *stocks -= amount);
                    seller.defined_stock_amount_price.0 -= amount;
                } else {
                    println!("Was unable to buy/sell {} stocks", amount);
                }

            // sell shares of company
            } else { 
                let cost = buyer.defined_stock_amount_price.1 * amount as f32;
                if buyer.bal >= cost && *seller.company_stock_owned.get(buyer_id).unwrap() >= amount {
                    buyer.bal -= cost;
                    seller.bal += cost;

                    // Update stock ownership
                    buyer.company_stock_owned.entry(buyer_id.to_string()).and_modify(|stocks| *stocks += amount);
                    seller.company_stock_owned.entry(buyer_id.to_string()).and_modify(|stocks| *stocks -= amount);
                    buyer.defined_stock_amount_price.0 += amount;
                } else {
                    println!("Was unable to buy/sell {} stocks", amount);
                }
            }

            // adjust stock price based on what was sold or bought
            if is_buying {
                //seller = self.adjust_stock_price(seller, amount, is_buying);
                adjust_stock_price(&mut seller, amount, is_buying);
            } else {
                //buyer = self.adjust_stock_price(buyer, amount, is_buying);
                adjust_stock_price(&mut buyer, amount, is_buying);
            }
            self.users.insert(buyer_id.to_string(), buyer);
            self.users.insert(seller_id.to_string(), seller);
        }
    }

    pub fn random_market_event(&mut self) {
        let mut rng = rand::rng();
        
        let type_of_event = rng.random_range(1.0..=100.0);
        match type_of_event {
            1.0..=20.0 => {
                (|| { 
                    let is_inflation = rng.random_bool(0.8);
                    let change_percent = exp_weighted_change(1.5);
                    let mut stock_value_change;

                    for user in self.users.values_mut() {
                        stock_value_change = (user.defined_stock_amount_price.1 * change_percent).round() / 100.0;
                        if is_inflation {
                            user.defined_stock_amount_price.1 -= stock_value_change;
                            println!("Inflation: stock price decreased for {} by {:.2}%", user.user_id, stock_value_change);
                        } else {
                            user.defined_stock_amount_price.1 += stock_value_change;
                            println!("Deflation: stock price increased for {} by {:.2}%", user.user_id, stock_value_change);
                        }
                    }
                })
                ();
            },

            20.0..=100.0 => { 
                (|| {
                    let rand_index = rng.random_range(1.0..=self.users.len() as f32);
                    if let Some(rand_user) = self.users.keys().nth(rand_index as usize).cloned() {
                        if let Some(user) = self.users.get_mut(&rand_user) {
                            let change_percent = exp_weighted_change(0.7);

                            let stock_value_change = (user.defined_stock_amount_price.1 * change_percent).round() / 100.0;

                            if rng.random_bool(0.5) {
                                user.defined_stock_amount_price.1 += stock_value_change;
                                println!("Company {} just increased stock price by {}%", rand_user, stock_value_change * 100.0);
                            } else {
                                user.defined_stock_amount_price.1 -= stock_value_change;
                                println!("Company {} just decreased stock price by {}%", rand_user, stock_value_change * 100.0);
                            }
                        }
                    }
                })
                (); 
            },

            _ => {
                println!("how the hecking heck did you get here!??!");
            }
        }
    }

    pub fn bankruptcy(&mut self, id: &str) {
        // return stocks back to company and adjust stock price for respective company
        if let Some(bankrupt_user) = self.users.get(id) {
            let bankrupt_stock = bankrupt_user.company_stock_owned.clone();

            for user in self.users.values_mut() {
                if let Some(amount) = bankrupt_stock.get(&user.user_id) {
                    user.defined_stock_amount_price.0 += amount;
                    adjust_stock_price(user, *amount, false);
                }
            }

            // Remove bankrupt user from users list
            self.users.remove(id);
        } else {
            return;
        }
        // removes listing of bankrupt user for other users
        for (_, user) in self.users.iter_mut() {
            user.company_stock_owned.remove(id);
        }
        println!("user {} has been removed!", id);
    }
}


// Adjust stock price based on market dynamics 
// should change based on how many buys of a share
// should also change based on number of things bought from user
fn adjust_stock_price(company: &mut User, amount: u32, is_buying: bool) {
    let mut price = company.defined_stock_amount_price.1;

    //let max_price = 100.0;
    //let midpoint: f32 = max_price / 2.0;
    let price_rate = 0.01;

    // Exponentially change the price
    if is_buying {
        //price = price + (amount as f32 * price_rate); // linear 
        //price = price * (1.0_f32 + price_rate).powf(amount as f32); // exponential
        price = price + ((1.0_f32 + amount as f32).log10() * price_rate); // logarithmic
        //price = max_price / (1.0_f32 + f32::consts::E.powf(-(amount as f32 - midpoint) * price_rate)) // Sigmoid (S-curve) doesn't work
    } else {
        //price = price - (amount as f32 * price_rate); // linear
        //price = price / (1.0_f32 + price_rate).powf(amount as f32); // exponential
        price = price - ((1.0_f32 + amount as f32).log10() * price_rate); // logarithmic
        //price = max_price / (1.0_f32 + f32::consts::E.powf(-(amount as f32 - midpoint) * price_rate)) // Sigmoid (S-curve) doesn't work
    }
    price = (price * 10000.0).round() / 10000.0; // round to sigfig 0.0001
    
    // Update the stock price for the company
    company.defined_stock_amount_price.1 = price;
    //company
}



fn exp_weighted_change(rate: f32) -> f32 {
    let exp_value = || {
        let mut rng = rand::rng();
        let uniform_random: f32 = rng.random();
        let exp_random = -uniform_random.ln() / rate;  // Inverse transform sampling
        exp_random
    };

    (exp_value() % 50.0) + 1.0
}


// --- end of stock market functionality ---

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjust_stock_price_5_false() {
        let mut market = StockMarket::new(); 
        let user_name = "1111";

        market.add_user(&user_name);

        let user = market.users.get_mut(user_name).unwrap();
        let amount = 5;
        let is_buying = false;

        let initial_price = user.defined_stock_amount_price.1;

        adjust_stock_price(user, amount, is_buying);

        let adjusted_price = user.defined_stock_amount_price.1;
        let expected_change = (1.0_f32 + amount as f32).log10() * 0.01;
        let expected_price = initial_price - expected_change;

        assert!((adjusted_price - expected_price).abs() < 0.0001, "Expected price to be around {}, got {}", expected_price, adjusted_price);
    }

    #[test]
    fn test_exp_weighted_change() {
        let rate = 1.0;
        let value = exp_weighted_change(rate);

        assert!(value >= 1.0 && value < 51.0);
    }

}
*/

use poise::serenity_prelude as serenity;
use crate::{Data, Error, Context};
use crate::utils::send_embed;

pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        enter_market(),
        buy_stock(),
        sell_stock(),
        random_event(),
        file_for_bankruptcy(),
        portfolio(),
    ]
}

// --- helpers ---

/// Resolve a Discord mention/tag like `@Username` or a raw user-id string into
/// a Discord `UserId`, then return its stringified u64 (what we use as the
/// stock-market key).
#[allow(unused)]
async fn resolve_company_id(ctx: Context<'_>, raw: &str) -> Result<String, Error> {
    // Poise already parses a `serenity::User` argument for us, but when we
    // receive it as a plain String we do it manually here.
    let trimmed = raw.trim_matches(|c: char| !c.is_numeric());
    if trimmed.is_empty() {
        return Err("Could not parse a user-id from that input.".into());
    }
    Ok(trimmed.to_string())
}

// --- commands ---

/// Enter the stock market by registering with the system
#[poise::command(prefix_command, slash_command)]
pub async fn enter_market(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.to_string();

    let mut market = ctx.data().market.lock().await;

    // Check if user is already registered
    if market.get_users().contains_key(&user_id) {
        send_embed(
            ctx,
            "❌ Already registered",
            "You're already registered in the stock market.",
            0xFF0000
        ).await?;
        return Ok(());
    }

    // Add user to the market
    market.add_user(&user_id);

    send_embed(
        ctx,
        "✅ Successfully joined market",
        "You've been registered in the stock market! You now have a starting balance of $10.00 and can begin trading.",
        0x00C853
    ).await?;

    Ok(())
}

/// Buy [amount] shares of another user's company stock
#[poise::command(prefix_command, slash_command)]
pub async fn buy_stock(
    ctx: Context<'_>,

    #[description = "User whose stock you want to buy"]
    company: serenity::User,

    #[description = "Number of shares to buy"]
    amount: u32,
) -> Result<(), Error> {
    let buyer_id  = ctx.author().id.to_string();
    let seller_id = company.id.to_string();

    if buyer_id == seller_id {
        send_embed(ctx, "❌ Invalid trade", "You cannot buy your own stock.", 0xFF0000).await?;
        return Ok(());
    }

    let mut market = ctx.data().market.lock().await;

    // Make sure both parties are registered
    if !market.get_users().contains_key(&buyer_id) {
        send_embed(ctx, "❌ Not registered",
            "You don't have a market account yet. Use `/join` first.", 0xFF0000).await?;
        return Ok(());
    }
    if !market.get_users().contains_key(&seller_id) {
        send_embed(ctx, "❌ Unknown company",
            "That user is not listed on the market.", 0xFF0000).await?;
        return Ok(());
    }

    let seller_price  = market.get_user_info(&seller_id).defined_stock_amount_price.1;
    let seller_avail  = market.get_user_info(&seller_id).defined_stock_amount_price.0;
    let buyer_balance = market.get_user_info(&buyer_id).bal;
    let total_cost    = seller_price * amount as f32;

    if buyer_balance < total_cost {
        send_embed(ctx, "❌ Insufficient funds",
            &format!("You need **${:.2}** but only have **${:.2}**.", total_cost, buyer_balance),
            0xFF0000).await?;
        return Ok(());
    }
    if seller_avail < amount {
        send_embed(ctx, "❌ Not enough shares",
            &format!("{} only has **{}** shares available.", company.name, seller_avail),
            0xFF0000).await?;
        return Ok(());
    }

    market.buy_stock(&buyer_id, &seller_id, amount);

    // TODO: db::record_transaction(&buyer_id, &seller_id, amount, total_cost, "buy").await?;

    let new_price = market.get_user_info(&seller_id).defined_stock_amount_price.1;
    send_embed(
        ctx,
        "✅ Purchase complete",
        &format!(
            "You bought **{}** share(s) of **{}** for **${:.2}**.\n\
             New share price: **${:.4}**",
            amount, company.name, total_cost, new_price
        ),
        0x00C853,
    ).await?;

    Ok(())
}

/// Sell [amount] of your own company's shares back to the market
#[poise::command(prefix_command, slash_command)]
pub async fn sell_stock(
    ctx: Context<'_>,

    #[description = "User who currently holds your shares (pass yourself to reclaim from market)"]
    holder: serenity::User,

    #[description = "Number of shares to sell"]
    amount: u32,
) -> Result<(), Error> {
    let seller_id = ctx.author().id.to_string();
    let holder_id = holder.id.to_string();

    let mut market = ctx.data().market.lock().await;

    if !market.get_users().contains_key(&seller_id) {
        send_embed(ctx, "❌ Not registered",
            "You don't have a market account yet. Use `/join` first.", 0xFF0000).await?;
        return Ok(());
    }

    // Check the holder actually owns enough of the seller's stock
    let shares_held = market
        .get_user_info(&holder_id)
        .company_stock_owned
        .get(&seller_id)
        .copied()
        .unwrap_or(0);

    if shares_held < amount {
        send_embed(ctx, "❌ Not enough shares",
            &format!("{} only holds **{}** of your shares.", holder.name, shares_held),
            0xFF0000).await?;
        return Ok(());
    }

    let price      = market.get_user_info(&seller_id).defined_stock_amount_price.1;
    let total_gain = price * amount as f32;

    market.sell_stock(&seller_id, &holder_id, amount);

    // TODO: db::record_transaction(&seller_id, &holder_id, amount, total_gain, "sell").await?;

    let new_price = market.get_user_info(&seller_id).defined_stock_amount_price.1;
    send_embed(
        ctx,
        "✅ Sale complete",
        &format!(
            "You sold **{}** share(s) for **${:.2}** total.\n\
             New share price: **${:.4}**",
            amount, total_gain, new_price
        ),
        0x00C853,
    ).await?;

    Ok(())
}

/// Trigger a random market event that shifts stock prices
#[poise::command(prefix_command, slash_command)]
pub async fn random_event(ctx: Context<'_>) -> Result<(), Error> {
    let mut market = ctx.data().market.lock().await;

    if market.get_users().is_empty() {
        send_embed(ctx, "📭 Empty market",
            "No users are registered yet.", 0xFFAA00).await?;
        return Ok(());
    }

    // Capture a snapshot of prices before the event so we can report deltas
    let before: std::collections::HashMap<String, f32> = market
        .get_users()
        .iter()
        .map(|(id, u)| (id.clone(), u.defined_stock_amount_price.1))
        .collect();

    market.random_market_event();

    let mut description = String::from("**Price changes this event:**\n");
    for (id, user) in market.get_users() {
        let old = before.get(id).copied().unwrap_or(0.0);
        let new = user.defined_stock_amount_price.1;
        let diff = new - old;
        let arrow = if diff >= 0.0 { "🟢 +" } else { "🔴 " };
        description.push_str(&format!(
            "<@{id}>: ${old:.4} → ${new:.4}  ({arrow}{diff:.4})\n"
        ));
    }

    // TODO: db::log_market_event(&description).await?;

    send_embed(ctx, "🎲 Market event occurred!", &description, 0x2196F3).await?;
    Ok(())
}

/// Wipe your account and return all outstanding shares to their issuers
#[poise::command(prefix_command, slash_command)]
pub async fn file_for_bankruptcy(ctx: Context<'_>) -> Result<(), Error> {
    use crate::utils::{send_confirmation_msg, user_choice_from_msg, ConfirmResult, create_embed, EXPIRE_TIME};

    let author    = ctx.author();
    let author_id = author.id.to_string();

    {
        let market = ctx.data().market.lock().await;
        if !market.get_users().contains_key(&author_id) {
            send_embed(ctx, "❌ Not registered",
                "You don't have a market account to liquidate.", 0xFF0000).await?;
            return Ok(());
        }
    }

    // Ask the user to confirm via DM buttons
    let embed = create_embed(
        "⚠️ Confirm bankruptcy",
        "Are you sure you want to file for bankruptcy?\n\
         **This is irreversible.** All your shares will be returned and \
         your account will be deleted.",
        0xFF6600,
    ).await?;

    let dm_result = send_confirmation_msg(ctx, author, embed).await;
    if dm_result.is_err() {
        send_embed(ctx, "❌ DMs disabled",
            "Please enable DMs from server members so I can confirm this action.", 0xFF0000).await?;
        return Ok(());
    }

    let choice = user_choice_from_msg(
        ctx,
        author.id,
        ctx.id().to_string(),
        EXPIRE_TIME,
    ).await;

    match choice {
        ConfirmResult::Yes => {
            let mut market = ctx.data().market.lock().await;
            market.bankruptcy(&author_id);

            // TODO: db::delete_user_account(&author_id).await?;

            send_embed(
                ctx,
                "🏳️ Bankruptcy filed",
                "Your account has been removed and all shares returned.",
                0x9E9E9E,
            ).await?;
        }
        ConfirmResult::No => {
            send_embed(ctx, "↩️ Cancelled", "Bankruptcy cancelled.", 0x9E9E9E).await?;
        }
        ConfirmResult::Timeout => {
            send_embed(ctx, "⏰ Timed out",
                "Confirmation window expired. No action taken.", 0x9E9E9E).await?;
        }
    }

    Ok(())
}

/// Show your current balance, stock price, and portfolio
#[poise::command(prefix_command, slash_command)]
pub async fn portfolio(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.to_string();
    let market  = ctx.data().market.lock().await;

    let user = match market.get_users().get(&user_id) {
        Some(u) => u,
        None => {
            send_embed(ctx, "❌ Not registered",
                "You don't have a market account yet. Use `/join` first.", 0xFF0000).await?;
            return Ok(());
        }
    };

    let mut owned = String::new();
    for (company_id, &amount) in &user.company_stock_owned {
        if amount > 0 {
            let price = market
                .get_users()
                .get(company_id)
                .map(|u| u.defined_stock_amount_price.1)
                .unwrap_or(0.0);
            owned.push_str(&format!(
                "<@{company_id}>: **{amount}** share(s) @ ${price:.4} each\n"
            ));
        }
    }
    if owned.is_empty() {
        owned = "*None*".to_string();
    }

    let description = format!(
        "**Balance:** ${:.2}\n\
         **Your share price:** ${:.4}\n\
         **Shares available:** {}\n\n\
         **Holdings:**\n{}",
        user.bal,
        user.defined_stock_amount_price.1,
        user.defined_stock_amount_price.0,
        owned,
    );

    send_embed(ctx, &format!("📊 {}'s Portfolio", ctx.author().name), &description, 0x7C4DFF).await?;
    Ok(())
}

