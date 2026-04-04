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


// --- Start of discord commands -----------------------------------------------

/// Show bank balance
#[poise::command(prefix_command, slash_command)]
pub async fn balance(ctx: Context<'_>) -> Result<(), Error> {
    let requester_user = ctx.author();
    let pool = get_pool();

    let discord_id = requester_user.id.get() as i64;
    let record = sqlx::query!(
        r#"
            SELECT bucks 
            FROM accounts 
            WHERE discord_id = ?
        "#, 
        discord_id
    )
    .fetch_optional(pool)
    .await?;

    match record {
        Some(r) => {
            println!("queryied bal from {} with balance value {}.", requester_user.tag(), r.bucks);
            
            send_msg(ctx, &format!("You have **{}** Mycuh bucks", r.bucks), false).await?;
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
    let base_id = ctx.id().to_string();

    let pool = get_pool();

    // check if the user being requested is the same user
    /*
    if &recipient_user == requester_user {
        send_msg(ctx, "Cannot transfer Mycuh Bucks to yourself.", true).await?;
        println!("User {} tried to transfer bucks to themselves", requester_user);

        return Ok(());
    }
    */

    let sender_id = requester_user.id.get() as i64;
    let recipient_id = recipient_user.id.get() as i64;

    let bal = sqlx::query!(
        r#"
            SELECT a1.bucks AS sender_bal, a2.bucks AS recipient_bal
            FROM accounts as a1
            INNER JOIN accounts as a2 
                ON a1.discord_id = ? AND a2.discord_id = ?;
        "#,
        sender_id,
        recipient_id
    )
    .fetch_optional(pool)
    .await?;

    let bal = match bal {
        Some(r) => r,
        None => {
            send_msg(ctx, "User does not exist in database.", true).await?;
            return Ok(());
        }
    };

    println!("Sender: {} has bal: {},\nRecpient: {} has bal: {}", requester_user.tag(), bal.sender_bal, recipient_user.tag(), bal.recipient_bal);

    // this assumes that the user exist in the database
    if bal.recipient_bal < request_amount as i64 {
        send_msg(ctx, &format!("User {} only has {} bucks.", recipient_user.mention(), bal.recipient_bal), true).await?;
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

            requester_user.mention(),
            request_amount,
            recipient_user.mention(),
            request_reason,
            EXPIRE_TIME.as_secs(),
        ));


    let mut msg = send_confirmation_msg(ctx, &recipient_user, embed.clone()).await?;

    /// edits user choice message
    fn edit_embed(
        requester_user: &serenity::User, 
        request_amount: u32, 
        recipient_user: serenity::User, 
        request_reason: String, 
        color: i32, 
        status: &str
    ) -> serenity::EditMessage {

        serenity::EditMessage::new()
        .embeds(vec![
            serenity::CreateEmbed::new()
                .title("Mycuh Bucks Transfer Request")
                .color(color)
                .description(format!(
                    "{} requested **{}** buck(s) from {}\n\
                    Reason: **{}**\n\
                    User Response: **{}**",
                    requester_user.mention(),
                    request_amount,
                    recipient_user.mention(),
                    request_reason,
                    status)
                )
        ])
        .components(vec![])
    }

    let result = user_choice_from_msg(ctx, requester_user.id, ctx.id().to_string(), EXPIRE_TIME).await;

    // checks users choice and handles it
    match result {
        ConfirmResult::Yes => {
            println!("User {} Accepted  ", recipient_user.tag());

            let loss = bal.sender_bal - request_amount as i64;
            let gain = bal.recipient_bal + request_amount as i64;
            sqlx::query!(
                r#"
                    UPDATE accounts 
                    SET bucks = CASE discord_id 
                        WHEN ? THEN ?
                        WHEN ? THEN ? 
                    END;
                "#,
                sender_id, loss,
                recipient_id, gain
            )
            .execute(pool)
            .await?;

            send_msg(ctx, &format!("Request from {} succeded with amount {}.", recipient_user.mention(), request_amount), true).await?;

            msg.edit(
                ctx,
                edit_embed(requester_user, request_amount, recipient_user, request_reason, GREEN_HIGHLIGHT, "Accepted  ")
            ).await?;
        }

        ConfirmResult::No => {
            println!("User {} Denied ❌", recipient_user.tag());

            send_msg(ctx, &format!("Request from {} declined.", recipient_user.mention()), true).await?;

            msg.edit(
                ctx,
                edit_embed(requester_user, request_amount, recipient_user, request_reason, RED_HIGHLIGHT, "Denied ❌")
            ).await?;
        }

        ConfirmResult::Timeout => {
            println!("User {} Timeout ⏱ ", recipient_user.tag());

            send_msg(ctx, &format!("Request from {} timed-out.", recipient_user.mention()), true).await?;

            msg.edit(
                ctx,
                edit_embed(requester_user, request_amount, recipient_user, request_reason, GRAY_HIGHLIGHT, "Expired ⏱ ")
            ).await?;
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

    let pool = get_pool();

    // check if the user being requested is the same user
    /*
    if &sending_user == taking_user {
        send_msg(ctx, "Cannot transfer Mycuh Bucks to yourself.", true).await?;
        println!("User {} tried to transfer bucks to themselves", sending_user);

        return Ok(());
    }
    */

    let embed = create_embed("Are you sure you want to transfer?", "", BLUE_HIGHLIGHT).await?;
    let msg = send_confirmation_msg(ctx, taking_user, embed).await?;
    let option = user_choice_from_msg(ctx, taking_user.id, ctx.id().to_string(), EXPIRE_TIME);
    match option.await {
        ConfirmResult::No | ConfirmResult::Timeout => { 
            return Ok(());
        }
        _ => {}
    };


    let sending_id = sending_user.id.get() as i64;
    let taking_id = taking_user.id.get() as i64;

    let bal = sqlx::query!(
        r#"
            SELECT a1.bucks AS sending_bal, a2.bucks AS taking_bal
            FROM accounts as a1
            INNER JOIN accounts as a2 
                ON a1.discord_id = ? AND a2.discord_id = ?;
        "#,
        sending_id,
        taking_id
    )
    .fetch_optional(pool)
    .await?;

    let bal = match bal {
        Some(r) => r,
        None => {
            send_msg(ctx, "User does not exist in database.", true).await?;
            return Ok(());
        }
    };

    println!("Sender: {} has bal: {},\nRecpient: {} has bal: {}", sending_user.tag(), bal.sending_bal, taking_user.tag(), bal.taking_bal);

    // this assumes that the user exist in the database
    if bal.sending_bal < request_amount as i64 {
        send_msg(ctx, &format!("User {} only has {} bucks.", taking_user.mention(), bal.taking_bal), true).await?;
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

            sending_user.mention(),
            request_amount,
            taking_user.mention(),
            request_reason,
            EXPIRE_TIME.as_secs(),
        ));


    // TODO !!! needs major fix, this should be the dm handle with generic message type call
    let mut msg = send_confirmation_msg(ctx, taking_user, embed.clone()).await?;

    // this function is repeat serveral times with just some string changes, could be fixed later
    /// edits user choice message
    fn edit_embed(
        sending_user: &serenity::User, 
        request_amount: u32, 
        taking_user: serenity::User, 
        request_reason: String, 
        color: i32, 
        status: &str
    ) -> serenity::EditMessage {

        serenity::EditMessage::new()
        .embeds(vec![
            serenity::CreateEmbed::new()
                .title("Bucks Transfer Request")
                .color(color)
                .description(format!(
                    "{} sent **{}** buck(s) to {}\n\
                    Reason: **{}**\n\
                    User Response: **{}**",
                    sending_user.mention(),
                    request_amount,
                    taking_user.mention(),
                    request_reason,
                    status)
                )
        ])
        .components(vec![])
    }

    let result = user_choice_from_msg (ctx, sending_user.id, ctx.id().to_string(), EXPIRE_TIME).await;

    // checks users choice and handles it
    match result {
        ConfirmResult::Yes => {
            println!("User {} Accepted  ", taking_user.tag());

            let loss = bal.sending_bal - request_amount as i64;
            let gain = bal.taking_bal + request_amount as i64;
            sqlx::query!(
                r#"
                    UPDATE accounts 
                    SET bucks = CASE discord_id 
                        WHEN ? THEN ?
                        WHEN ? THEN ? 
                    END;
                "#,
                sending_id, loss,
                taking_id, gain
            )
            .execute(pool)
            .await?;

            send_msg(ctx, &format!("Request from {} succeded with amount {}.", taking_user.mention(), request_amount), true).await?;

            msg.edit(
                ctx,
                edit_embed(taking_user, request_amount, sending_user, request_reason, GREEN_HIGHLIGHT, "Accepted  ")
            ).await?;
        }

        ConfirmResult::No => {
            println!("User {} Denied ❌", taking_user.tag());

            send_msg(ctx, &format!("Request from {} declined.", taking_user.mention()), true).await?;

            msg.edit(
                ctx,
                edit_embed(taking_user, request_amount, sending_user, request_reason, RED_HIGHLIGHT, "Denied ❌")
            ).await?;
        }

        ConfirmResult::Timeout => {
            println!("User {} Timeout ⏱ ", taking_user.tag());

            send_msg(ctx, &format!("Request from {} timed-out.", taking_user.mention()), true).await?;

            msg.edit(
                ctx,
                edit_embed(taking_user, request_amount, sending_user, request_reason, GRAY_HIGHLIGHT, "Expired ⏱ ")
            ).await?;
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
    let base_id = ctx.id().to_string();
    let user = ctx.author();
    let user_id = user.id.get() as i64;
    let owner_id = crate::utils::USER_OWNER; // need some api to get
    let owner_user = owner_id.to_user(ctx.serenity_context()).await?;

    let pool = get_pool(); 

    let embed = serenity::CreateEmbed::default()
        .title("Mycuh Bucks Transfer Request")
        .color(GRAY_HIGHLIGHT)
        .description(format!(
            "{} is requesting **{}** mycuh buck(s)\n\
            Reason: **{}**\n\
            User Response: **{} secs** ⏳",

            user.id.mention(),
            request_amount,
            request_reason,
            EXPIRE_TIME.as_secs(),
        ));

    let bal = sqlx::query!(
        r#"
            SELECT bucks
            FROM accounts
            WHERE discord_id = ?;
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await?;

    let bal = match bal {
        Some(r) => r,
        None => {
            ask_user_join_db(ctx).await?;
            return Ok(());
        }
    };
    // recpient dm hook to send/edit messages
    // ??? not sure how to send dm to specific user
    let dm = match owner_user.create_dm_channel(ctx.serenity_context()).await {
        Ok(dm) => dm,
        Err(_) => {
            send_msg(ctx, "That user has DMs disabled.", true).await?;
            return Ok(());
        }
    };

    let mut msg = send_confirmation_msg(ctx, &owner_user, embed.clone()).await?;

    /// edits user choice message
    fn edit_embed(
        user: &serenity::User, 
        request_amount: u32, 
        owner_user: serenity::User, 
        request_reason: String, 
        color: i32, 
        status: &str
    ) -> serenity::EditMessage {

        serenity::EditMessage::new()
        .embeds(vec![
            serenity::CreateEmbed::new()
                .title("Bucks Transfer Request")
                .color(color)
                .description(format!(
                    "{} requested **{}** buck(s) from {}\n\
                    Reason: **{}**\n\
                    User Response: **{}**",
                    user.mention(),
                    request_amount,
                    owner_user.mention(),
                    request_reason,
                    status)
                )
        ])
        .components(vec![])
    }

    let result = user_choice_from_msg(ctx, owner_id, ctx.id().to_string(), EXPIRE_TIME).await;

    match result {
        ConfirmResult::Yes => {
            let gain = bal.bucks + request_amount as i64;
            sqlx::query!(
                r#"
                    UPDATE accounts 
                    SET bucks = CASE discord_id 
                        WHEN ? THEN ? 
                    END;
                "#,
                user_id, gain
            )
            .execute(pool)
            .await?;

            send_msg(ctx, &format!("Request from {} succeded with amount {}.", owner_user.mention(), request_amount), true).await?;

            msg.edit(
                ctx,
                edit_embed(user, request_amount, owner_user, request_reason, GREEN_HIGHLIGHT, "Accepted  ")
            ).await?;
        }

        ConfirmResult::No => {
            send_msg(ctx, &format!("Request from {} declined.", owner_user.mention()), true).await?;

            msg.edit(
                ctx,
                edit_embed(user, request_amount, owner_user, request_reason, RED_HIGHLIGHT, "Denied ❌")
            ).await?;
        }

        ConfirmResult::Timeout => {
            println!("User {} Timeout ⏱ ", owner_user.tag());

            send_msg(ctx, &format!("Request from {} timed-out.", owner_user.mention()), true).await?;

            msg.edit(
                ctx,
                edit_embed(user, request_amount, owner_user, request_reason, GRAY_HIGHLIGHT, "Expired ⏱ ")
            ).await?;
        }
    }

    
    Ok(())
}



/// List the leaderboard of most the most wealthy users
#[poise::command(prefix_command, slash_command)]
pub async fn fat_cat_stats(ctx: Context<'_>,) -> Result<(), Error> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    let pool = get_pool();

    let records = sqlx::query!(
        r#"
            SELECT username, bucks
            FROM accounts
            ORDER BY bucks DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

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
    let end_time = Instant::now();

    let delta = (end_time - start_time).as_micros();

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
    let user = ctx.author();
    let discord_id = user.id.get() as i64;
    
    let pool = get_pool(); 

    send_msg(ctx, format!("Are you sure you want to add **{}** to your product listing?", product_name).as_str(), false).await?;
    let option = user_choice_from_msg(ctx, user.id, ctx.id().to_string(), EXPIRE_TIME);
    match option.await {
        ConfirmResult::No | ConfirmResult::Timeout => { 
            return Ok(());
        }
        _ => {}
    };

    sqlx::query!(
        r#"
            INSERT INTO account_product_listings (discord_id, product_name, product_price, product_description)
            VALUES (?, ?, ?, ?);
        "#,
        discord_id, product_name, product_price, product_description
    )
    .execute(pool)
    .await?;
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
    let user = ctx.author();
    let discord_id = user.id.get() as i64;
    
    let pool = get_pool(); 

    send_msg(ctx, format!("Are you sure you want to remove **{}** from your listing?", product_name).as_str(), true).await?;
    let option = user_choice_from_msg(ctx, user.id, ctx.id().to_string(), EXPIRE_TIME);
    match option.await {
        ConfirmResult::No | ConfirmResult::Timeout => { 
            return Ok(());
        }
        _ => {}
    };

    let deleted = sqlx::query!(
        r#"
            DELETE FROM account_product_listings 
            WHERE discord_id = ? AND product_name = ?
            RETURNING *;
        "#,
        discord_id, product_name
    )
    .fetch_optional(pool)
    .await?;

    match deleted {
        Some(deleted) => { 
            let product_description = match deleted.product_description {
                Some(pd) => format!("\nDescription: **{}**", pd),
                None => "".to_string()
            };

            send_msg(ctx, format!("
                Removed **{}** from your product page.\n
                Price: **{}** {}", 
                deleted.product_name.unwrap(),
                deleted.product_price,
                product_description,
            ).as_str(),
            true)
            .await?; 
        }
        None => { send_msg(ctx, format!("**{}** doesn't exist in your product page.", product_name).as_str(), true).await?; }
    }

    Ok(())
}


/// View a discord user product listing
#[poise::command(prefix_command, slash_command)]
pub async fn view_product_listing(
    ctx: Context<'_>,

    #[description = "Member to view listings from"]
    discord_user: serenity::User, // change to reponse from emoji from user
) -> Result<(), Error> {
    let discord_id = discord_user.id.get() as i64;

    let pool = get_pool();

    let list = sqlx::query!(
        r#"
            SELECT * FROM account_product_listings 
            WHERE discord_id = ?;
        "#,
        discord_id
    )
    .fetch_optional(pool)
    .await?;

    match list {
        Some(user_list) => { 
            //println!("{:#?}", record);
            send_msg(ctx, format!("**{}** product page.\n{:#?}", discord_user.mention(), user_list).as_str(), true).await?; 
        }
        None => { send_msg(ctx, format!("**{}** doesn't have any listings or doesn't exist in database.", discord_user.mention()).as_str(), true).await?; }

    }

    Ok(())
}

