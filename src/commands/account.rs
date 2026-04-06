use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;

#[allow(unused_imports)]
use crate::{Data, Error, Context};
use crate::db::*;
use crate::utils::*;

pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        balance(),
        transfer_request(),
        transfer_send(),
        request_loan(),
        fat_cat_stats(),
        add_product_listing(),
        remove_product_listing(),
        view_product_listing(),
    ]
}


// --- Shared embed helpers ----------------------------------------------------

fn result_embed(
    title: &str,
    first_line: &str,
    reason: &str,
    color: i32,
    status: &str,
) -> serenity::EditMessage {
    serenity::EditMessage::new()
        .embeds(vec![
            serenity::CreateEmbed::new()
                .title(title)
                .color(color)
                .description(format!(
                    "{}\nReason: **{}**\nUser Response: **{}**",
                    first_line, reason, status
                ))
        ])
        .components(vec![])
}

// --- Start of discord commands -----------------------------------------------

/// Show bank balance
#[poise::command(prefix_command, slash_command)]
pub async fn balance(ctx: Context<'_>) -> Result<(), Error> {
    let requester_user = ctx.author();
    let discord_id = requester_user.id.get() as i64;

    match fetch_balance(discord_id).await? {
        Some(bucks) => {
            println!("queryied bal from {} with balance value {}.", requester_user.tag(), bucks);
            send_msg(ctx, &format!("You have **{}** Mycuh bucks", bucks), false).await?;
        }
        None => {
            let added_to_db = ask_user_join_db(ctx).await?;
            if added_to_db != ConfirmResult::Yes {
                send_msg(ctx, "You have **0** Mycuh bucks", false).await?;
            }
            return Ok(())
        }
    };
    Ok(())
}



/// Ask another member to transfer mycuh buck(s) to themselves
#[poise::command(prefix_command, slash_command)]
pub async fn transfer_request(
    ctx: Context<'_>,

    #[description = "Amount to transfer"]
    request_amount: u32,

    #[description = "Reason for transfer"]
    request_reason: String,

    #[description = "Member to transfer to/from"]
    recipient_user: serenity::User,
) -> Result<(), Error> {
    let requester_user = ctx.author();
    let _base_id = ctx.id().to_string();

    let sender_id    = requester_user.id.get() as i64;
    let recipient_id = recipient_user.id.get() as i64;

    let (sender_bal, recipient_bal) = match fetch_two_balances(sender_id, recipient_id).await? {
        Some(b) => b,
        None => {
            send_msg(ctx, "User does not exist in database.", true).await?;
            return Ok(());
        }
    };

    println!("Sender: {} has bal: {},\nRecpient: {} has bal: {}", requester_user.tag(), sender_bal, recipient_user.tag(), recipient_bal);

    if recipient_bal < request_amount as i64 {
        send_msg(ctx, &format!("User {} only has {} bucks.", recipient_user.mention(), recipient_bal), true).await?;
        println!("User {} does not have enough Mycuh bucks to give.", requester_user.tag());
        return Ok(());
    }

    send_msg(ctx, &format!("Sent transfer request to {} time till expire in {}s.", recipient_user.mention(), EXPIRE_TIME.as_secs()), true).await?;

    let embed = serenity::CreateEmbed::default()
        .title("Mycuh Bucks Transfer Request")
        .color(GRAY_HIGHLIGHT)
        .description(format!(
            "{} is requesting to add **{}** mycuh buck(s) from {} to their myccount\n\
            Reason: **{}**\n\
            User Response: **{} secs** ⏳",
            requester_user.mention(), request_amount, recipient_user.mention(),
            request_reason, EXPIRE_TIME.as_secs(),
        ));

    let mut msg = send_confirmation_msg(ctx, &recipient_user, embed).await?;
    let result  = user_choice_from_msg(ctx, requester_user.id, ctx.id().to_string(), EXPIRE_TIME).await;

    let first_line = format!(
        "{} requested **{}** buck(s) from {}",
        requester_user.mention(), request_amount, recipient_user.mention()
    );

    match result {
        ConfirmResult::Yes => {
            println!("User {} Accepted  ", recipient_user.tag());

            let loss = sender_bal    - request_amount as i64;
            let gain = recipient_bal + request_amount as i64;
            update_two_balances(sender_id, loss, recipient_id, gain).await?;

            send_msg(ctx, &format!("Request from {} succeded with amount {}.", recipient_user.mention(), request_amount), true).await?;
            msg.edit(ctx, result_embed("Mycuh Bucks Transfer Request", &first_line, &request_reason, GREEN_HIGHLIGHT, "Accepted  ")).await?;
        }

        ConfirmResult::No => {
            println!("User {} Denied ❌", recipient_user.tag());
            send_msg(ctx, &format!("Request from {} declined.", recipient_user.mention()), true).await?;
            msg.edit(ctx, result_embed("Mycuh Bucks Transfer Request", &first_line, &request_reason, RED_HIGHLIGHT, "Denied ❌")).await?;
        }

        ConfirmResult::Timeout => {
            println!("User {} Timeout ⏱ ", recipient_user.tag());
            send_msg(ctx, &format!("Request from {} timed-out.", recipient_user.mention()), true).await?;
            msg.edit(ctx, result_embed("Mycuh Bucks Transfer Request", &first_line, &request_reason, GRAY_HIGHLIGHT, "Expired ⏱ ")).await?;
        }
    }

    Ok(())
}


/// Ask another member to transfer mycuh buck(s) to them
#[poise::command(prefix_command, slash_command)]
pub async fn transfer_send(
    ctx: Context<'_>,

    #[description = "Amount to transfer"]
    request_amount: u32,

    #[description = "Reason for transfer"]
    request_reason: String,

    #[description = "Member to transfer to/from"]
    sending_user: serenity::User,
) -> Result<(), Error> {
    let taking_user = ctx.author();

    let embed = create_embed("Are you sure you want to transfer?", "", BLUE_HIGHLIGHT).await?;
    let _msg  = send_confirmation_msg(ctx, taking_user, embed).await?;
    match user_choice_from_msg(ctx, taking_user.id, ctx.id().to_string(), EXPIRE_TIME).await {
        ConfirmResult::No | ConfirmResult::Timeout => return Ok(()),
        _ => {}
    };

    let sending_id = sending_user.id.get() as i64;
    let taking_id  = taking_user.id.get()  as i64;

    let (sending_bal, taking_bal) = match fetch_two_balances(sending_id, taking_id).await? {
        Some(b) => b,
        None => {
            send_msg(ctx, "User does not exist in database.", true).await?;
            return Ok(());
        }
    };

    println!("Sender: {} has bal: {},\nRecpient: {} has bal: {}", sending_user.tag(), sending_bal, taking_user.tag(), taking_bal);

    if sending_bal < request_amount as i64 {
        send_msg(ctx, &format!("User {} only has {} bucks.", taking_user.mention(), taking_bal), true).await?;
        println!("User {} does not have enough Mycuh bucks to give.", sending_user.tag());
        return Ok(());
    }

    send_msg(ctx, &format!("Sent transfer request to {} time till expire {}.", taking_user.mention(), EXPIRE_TIME.as_secs()), true).await?;

    let embed = serenity::CreateEmbed::default()
        .title("Mycuh Bucks Transfer Request")
        .color(GRAY_HIGHLIGHT)
        .description(format!(
            "{} is requesting to give **{}** mycuh buck(s) to {} myccount\n\
            Reason: **{}**\n\
            User Response: **{} secs** ⏳",
            sending_user.mention(), request_amount, taking_user.mention(),
            request_reason, EXPIRE_TIME.as_secs(),
        ));

    let mut msg = send_confirmation_msg(ctx, taking_user, embed).await?;
    let result  = user_choice_from_msg(ctx, sending_user.id, ctx.id().to_string(), EXPIRE_TIME).await;

    let first_line = format!(
        "{} sent **{}** buck(s) to {}",
        taking_user.mention(), request_amount, sending_user.mention()
    );

    match result {
        ConfirmResult::Yes => {
            println!("User {} Accepted  ", taking_user.tag());

            let loss = sending_bal - request_amount as i64;
            let gain = taking_bal  + request_amount as i64;
            update_two_balances(sending_id, loss, taking_id, gain).await?;

            send_msg(ctx, &format!("Request from {} succeded with amount {}.", taking_user.mention(), request_amount), true).await?;
            msg.edit(ctx, result_embed("Bucks Transfer Request", &first_line, &request_reason, GREEN_HIGHLIGHT, "Accepted  ")).await?;
        }

        ConfirmResult::No => {
            println!("User {} Denied ❌", taking_user.tag());
            send_msg(ctx, &format!("Request from {} declined.", taking_user.mention()), true).await?;
            msg.edit(ctx, result_embed("Bucks Transfer Request", &first_line, &request_reason, RED_HIGHLIGHT, "Denied ❌")).await?;
        }

        ConfirmResult::Timeout => {
            println!("User {} Timeout ⏱ ", taking_user.tag());
            send_msg(ctx, &format!("Request from {} timed-out.", taking_user.mention()), true).await?;
            msg.edit(ctx, result_embed("Bucks Transfer Request", &first_line, &request_reason, GRAY_HIGHLIGHT, "Expired ⏱ ")).await?;
        }
    }

    Ok(())
}



/// Request amount of mycuh bucks from the mycuh bank
#[allow(dead_code, unused_variables)]
#[poise::command(prefix_command, slash_command)]
pub async fn request_loan(
    ctx: Context<'_>,

    #[description = "Amount of bucks to request"]
    request_amount: u32,

    #[description = "Reason for the request"]
    request_reason: String,
) -> Result<(), Error> {
    let base_id  = ctx.id().to_string();
    let user     = ctx.author();
    let user_id  = user.id.get() as i64;
    let owner_id = crate::utils::USER_OWNER;
    let owner_user = owner_id.to_user(ctx.serenity_context()).await?;

    let embed = serenity::CreateEmbed::default()
        .title("Mycuh Bucks Transfer Request")
        .color(GRAY_HIGHLIGHT)
        .description(format!(
            "{} is requesting **{}** mycuh buck(s)\n\
            Reason: **{}**\n\
            User Response: **{} secs** ⏳",
            user.id.mention(), request_amount, request_reason, EXPIRE_TIME.as_secs(),
        ));

    let bucks = match fetch_balance(user_id).await? {
        Some(b) => b,
        None => {
            ask_user_join_db(ctx).await?;
            return Ok(());
        }
    };

    let dm = match owner_user.create_dm_channel(ctx.serenity_context()).await {
        Ok(dm) => dm,
        Err(_) => {
            send_msg(ctx, "That user has DMs disabled.", true).await?;
            return Ok(());
        }
    };

    let mut msg = send_confirmation_msg(ctx, &owner_user, embed).await?;
    let result  = user_choice_from_msg(ctx, owner_id, ctx.id().to_string(), EXPIRE_TIME).await;

    let first_line = format!(
        "{} requested **{}** buck(s) from {}",
        user.mention(), request_amount, owner_user.mention()
    );

    match result {
        ConfirmResult::Yes => {
            let gain = bucks + request_amount as i64;
            update_balance(user_id, gain).await?;

            send_msg(ctx, &format!("Request from {} succeded with amount {}.", owner_user.mention(), request_amount), true).await?;
            msg.edit(ctx, result_embed("Bucks Transfer Request", &first_line, &request_reason, GREEN_HIGHLIGHT, "Accepted  ")).await?;
        }

        ConfirmResult::No => {
            send_msg(ctx, &format!("Request from {} declined.", owner_user.mention()), true).await?;
            msg.edit(ctx, result_embed("Bucks Transfer Request", &first_line, &request_reason, RED_HIGHLIGHT, "Denied ❌")).await?;
        }

        ConfirmResult::Timeout => {
            println!("User {} Timeout ⏱ ", owner_user.tag());
            send_msg(ctx, &format!("Request from {} timed-out.", owner_user.mention()), true).await?;
            msg.edit(ctx, result_embed("Bucks Transfer Request", &first_line, &request_reason, GRAY_HIGHLIGHT, "Expired ⏱ ")).await?;
        }
    }

    Ok(())
}



/// List the leaderboard of most the most wealthy users
#[poise::command(prefix_command, slash_command)]
pub async fn fat_cat_stats(ctx: Context<'_>,) -> Result<(), Error> {
    use std::time::Instant;

    let start_time = Instant::now();

    let records = fetch_leaderboard().await?;

    fn medal(rank: usize) -> &'static str {
        match rank {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "",
        }
    }

    let rankings = records
        .iter()
        .take(10)
        .enumerate()
        .map(|(i, row)| {
            format!(
                "**#{}**. **{}** -- **{}** bucks {}\n",
                i + 1,
                row.username,
                row.bucks,
                medal(i)
            )
        })
        .collect::<String>();

    let delta = (Instant::now() - start_time).as_micros();
    println!("Took {} microsecs to compute Mycuh Buck Stats", delta);

    send_embed(ctx, "**BIG CAT STATS**", &rankings, BLUE_HIGHLIGHT).await?;

    Ok(())
}




// --- personal listings commands ---

/// Add a new product to listing
#[poise::command(prefix_command, slash_command)]
pub async fn add_product_listing(
    ctx: Context<'_>,

    #[description = "Name Of Product"]
    product_name: String,

    #[description = "Description Of Product"]
    product_description: String,

    #[description = "Amount of bucks to request"]
    product_price: u32,
) -> Result<(), Error> {
    let user       = ctx.author();
    let discord_id = user.id.get() as i64;

    send_msg(ctx, format!("Are you sure you want to add **{}** to your product listing?", product_name).as_str(), false).await?;
    match user_choice_from_msg(ctx, user.id, ctx.id().to_string(), EXPIRE_TIME).await {
        ConfirmResult::No | ConfirmResult::Timeout => return Ok(()),
        _ => {}
    };

    create_product(discord_id, &product_name, product_price, &product_description).await?;
    send_msg(ctx, format!("Added **{}** to your product page.", product_name).as_str(), true).await?;

    Ok(())
}



/// Remove product from listing
#[poise::command(prefix_command, slash_command)]
pub async fn remove_product_listing(
    ctx: Context<'_>,

    #[description = "Name Of Product"]
    product_name: String,
) -> Result<(), Error> {
    let user       = ctx.author();
    let discord_id = user.id.get() as i64;

    send_msg(ctx, format!("Are you sure you want to remove **{}** from your listing?", product_name).as_str(), true).await?;
    match user_choice_from_msg(ctx, user.id, ctx.id().to_string(), EXPIRE_TIME).await {
        ConfirmResult::No | ConfirmResult::Timeout => return Ok(()),
        _ => {}
    };

    match remove_product(discord_id, &product_name).await? {
        Some(listing) => {
            let description_line = match listing.product_description {
                Some(pd) => format!("\nDescription: **{}**", pd),
                None => "".to_string(),
            };

            send_msg(ctx, format!("
                Removed **{}** from your product page.\n
                Price: **{}** {}",
                listing.product_name.unwrap(),
                listing.product_price,
                description_line,
            ).as_str(), true).await?;
        }
        None => {
            send_msg(ctx, format!("**{}** doesn't exist in your product page.", product_name).as_str(), true).await?;
        }
    }

    Ok(())
}


/// View a discord user product listing
#[poise::command(prefix_command, slash_command)]
pub async fn view_product_listing(
    ctx: Context<'_>,

    #[description = "Member to view listings from"]
    discord_user: serenity::User,
) -> Result<(), Error> {
    let discord_id = discord_user.id.get() as i64;

    match fetch_product(discord_id).await? {
        Some(listing) => {
            send_msg(ctx, format!("**{}** product page.\n{:#?}", discord_user.mention(), listing).as_str(), true).await?;
        }
        None => {
            send_msg(ctx, format!("**{}** doesn't have any listings or doesn't exist in database.", discord_user.mention()).as_str(), true).await?;
        }
    }

    Ok(())
}
