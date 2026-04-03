use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;

#[allow(unused_imports)]
use crate::{Data, Error, Context};
use crate::{get_pool, init_user_account};

const GRAY_HIGHLIGHT: i32 = 0x747F8D;
const GREEN_HIGHLIGHT: i32 = 0x57F287;
const RED_HIGHLIGHT: i32 = 0xED4245;
//const EXPIRE_TIME: std::time::Duration = std::time::Duration::from_secs(300);
const EXPIRE_TIME: std::time::Duration = std::time::Duration::from_secs(15);

pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        balance(),
        transfer_request(),
        transfer_send(),
        //request_loan(),
        add_product_to_listing(),
        remove_product_in_listing(),
        view_listing(),
    ]
}



/// Sends a hidden message 'msg' to the user from 'ctx' 
async fn send_hidden_msg<'a>(ctx: Context<'a>, msg: &str) -> Result<poise::ReplyHandle<'a>, Error> {
    let reply = ctx.send(poise::CreateReply::default()
        .content(msg)
        .ephemeral(true)
    )
    .await?;

    Ok(reply)
}


/// user button choice component help function
fn emoji_buttons_components(base_id: &String) -> Vec<serenity::CreateActionRow> {
    let btn_yes = serenity::CreateButton::new(format!("{base_id}:yes"))
        .style(serenity::ButtonStyle::Secondary)
        .label("Yes")
        .emoji('✅');

    let btn_no = serenity::CreateButton::new(format!("{base_id}:no"))
        .style(serenity::ButtonStyle::Secondary)
        .label("No")
        .emoji('❌');

    vec![serenity::CreateActionRow::Buttons(vec![btn_yes, btn_no,] )]
}

    
#[derive(PartialEq)]
pub enum ConfirmResult {
    Yes,
    No,
    Timeout,
}

pub async fn user_choice_from_msg (
    ctx: Context<'_>,
    author_id: serenity::UserId,
    base_id: String,
    timeout: std::time::Duration,
) -> ConfirmResult {
    // recpient selection data [yes/no/timeout] 
    let interaction = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(author_id)
        .timeout(timeout)
        .filter(move |mci| {
            mci.data.custom_id == format!("{base_id}:yes") 
                || mci.data.custom_id == format!("{base_id}:no")
        }
    ).await;

    match interaction {
        Some(mci) => {
            if mci.data.custom_id.ends_with(":yes") {
                ConfirmResult::Yes
            } else if mci.data.custom_id.ends_with(":no") {
                ConfirmResult::No
            } else {
                panic!("interaction result wasn't [yes/no/timeout]");
            } 

        },

        None => ConfirmResult::Timeout,
    }
}

pub async fn send_option_message(
    ctx: Context<'_>,
    channel: &serenity::ChannelId,
    embed: serenity::CreateEmbed,
    base_id: &String,
) -> Result<serenity::Message, serenity::Error> {
    channel
        .send_message(
            ctx,
            serenity::CreateMessage::new()
                .embed(embed)
                .components(emoji_buttons_components(&base_id))
        )
        .await
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
            
            send_hidden_msg(ctx, &format!("You have **{}** Mycuh bucks", r.bucks) ).await?;
        }
        None => {
            let added_to_db = ask_user_join_db(ctx).await?;
            if added_to_db != ConfirmResult::Yes {
                send_hidden_msg(ctx, "You have **0** Mycuh bucks").await?;
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
    if &recipient_user == requester_user {
        send_hidden_msg(ctx, "Cannot transfer Mycuh Bucks to yourself.").await?;
        println!("User {} tried to transfer bucks to themselves", requester_user);

        return Ok(());
    }

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
            send_hidden_msg(ctx, "User does not exist in database.").await?;
            return Ok(());
        }
    };

    println!("Sender: {} has bal: {},\nRecpient: {} has bal: {}", requester_user.tag(), bal.sender_bal, recipient_user.tag(), bal.recipient_bal);

    // this assumes that the user exist in the database
    if bal.recipient_bal < request_amount as i64 {
        send_hidden_msg(ctx, &format!("User {} only has {} bucks.", recipient_user.mention(), bal.recipient_bal)).await?;
        println!("User {} does not have enough Mycuh bucks to give.", requester_user.tag());

        return Ok(());
    }
    send_hidden_msg(ctx, &format!("Sent transfer request to {} time till expire {}.", recipient_user.mention(), EXPIRE_TIME.as_secs()) ).await?;


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

    // recpient dm hook to send/edit messages
    let dm = match recipient_user.create_dm_channel(ctx.serenity_context()).await {
        Ok(dm) => dm,
        Err(_) => {
            send_hidden_msg(ctx, "That user has DMs disabled.").await?;
            return Ok(());
        }
    };


    // TODO !!! needs major fix, this should be the dm handle with generic message type call
    let mut msg = send_option_message(ctx, &dm.id, embed.clone(), &base_id).await?;


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
                .title("Bucks Transfer Request")
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

    let result = user_choice_from_msg (ctx, requester_user.id, base_id.clone(), EXPIRE_TIME).await;

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

            send_hidden_msg(ctx, &format!("Request from {} succeded with amount {}.", recipient_user.mention(), request_amount) ).await?;

            msg.edit(
                ctx,
                edit_embed(requester_user, request_amount, recipient_user, request_reason, GREEN_HIGHLIGHT, "Accepted  ")
            ).await?;
        }

        ConfirmResult::No => {
            println!("User {} Denied ❌", recipient_user.tag());

            send_hidden_msg(ctx, &format!("Request from {} declined.", recipient_user.mention()) ).await?;

            msg.edit(
                ctx,
                edit_embed(requester_user, request_amount, recipient_user, request_reason, RED_HIGHLIGHT, "Denied ❌")
            ).await?;
        }

        ConfirmResult::Timeout => {
            println!("User {} Timeout ⏱ ", recipient_user.tag());

            send_hidden_msg(ctx, &format!("Request from {} timed-out.", recipient_user.mention()) ).await?;

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
    let base_id = ctx.id().to_string();

    let pool = get_pool();

    // check if the user being requested is the same user
    if &sending_user == taking_user {
        send_hidden_msg(ctx, "Cannot transfer Mycuh Bucks to yourself.").await?;
        println!("User {} tried to transfer bucks to themselves", sending_user);

        return Ok(());
    }

    send_hidden_msg(ctx, "Are you sure you want to transfer?").await?;
    let option = user_choice_from_msg(ctx, ctx.author().id, ctx.id().to_string(), EXPIRE_TIME);
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
            send_hidden_msg(ctx, "User does not exist in database.").await?;
            return Ok(());
        }
    };

    println!("Sender: {} has bal: {},\nRecpient: {} has bal: {}", sending_user.tag(), bal.sending_bal, taking_user.tag(), bal.taking_bal);

    // this assumes that the user exist in the database
    if bal.sending_bal < request_amount as i64 {
        send_hidden_msg(ctx, &format!("User {} only has {} bucks.", taking_user.mention(), bal.taking_bal)).await?;
        println!("User {} does not have enough Mycuh bucks to give.", sending_user.tag());

        return Ok(());
    }
    send_hidden_msg(ctx, &format!("Sent transfer request to {} time till expire {}.", taking_user.mention(), EXPIRE_TIME.as_secs()) ).await?;


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

    // recpient dm hook to send/edit messages
    let dm = match taking_user.create_dm_channel(ctx.serenity_context()).await {
        Ok(dm) => dm,
        Err(_) => {
            send_hidden_msg(ctx, "That user has DMs disabled.").await?;
            return Ok(());
        }
    };


    // TODO !!! needs major fix, this should be the dm handle with generic message type call
    let mut msg = send_option_message(ctx, &dm.id, embed.clone(), &base_id).await?;


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
                    "{} requested **{}** buck(s) from {}\n\
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

    let result = user_choice_from_msg (ctx, sending_user.id, base_id.clone(), EXPIRE_TIME).await;

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

            send_hidden_msg(ctx, &format!("Request from {} succeded with amount {}.", taking_user.mention(), request_amount) ).await?;

            msg.edit(
                ctx,
                edit_embed(taking_user, request_amount, sending_user, request_reason, GREEN_HIGHLIGHT, "Accepted  ")
            ).await?;
        }

        ConfirmResult::No => {
            println!("User {} Denied ❌", taking_user.tag());

            send_hidden_msg(ctx, &format!("Request from {} declined.", taking_user.mention()) ).await?;

            msg.edit(
                ctx,
                edit_embed(taking_user, request_amount, sending_user, request_reason, RED_HIGHLIGHT, "Denied ❌")
            ).await?;
        }

        ConfirmResult::Timeout => {
            println!("User {} Timeout ⏱ ", taking_user.tag());

            send_hidden_msg(ctx, &format!("Request from {} timed-out.", taking_user.mention()) ).await?;

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
    request_amount: i32,

    #[description = "Reason for the request"]
    request_reason: String,

    #[description = "User you are requesting bucks from"]
    user: serenity::User, // change to reponse from emoji from user
) -> Result<(), Error> {
    // sql query
    
    
    Ok(())
}





// --- personal listings commands ---

/// Add a new product to listing
#[poise::command(prefix_command, slash_command)]
pub async fn add_product_to_listing(
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

    send_hidden_msg(ctx, format!("Are you sure you want to add **{}** to your product listing?", product_name).as_str() ).await?;
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
    send_hidden_msg(ctx, format!("Added **{}** to your product page.", product_name).as_str() ).await?;
    
    Ok(())
}



/// Remove product from listing
#[allow(dead_code, unused_variables)]
#[poise::command(prefix_command, slash_command)]
pub async fn remove_product_in_listing(
    ctx: Context<'_>,

    #[description = "Name Of Product"]
    product_name: String,
) -> Result<(), Error> {
    let user = ctx.author();
    let discord_id = user.id.get() as i64;
    
    let pool = get_pool(); 

    send_hidden_msg(ctx, format!("Are you sure you want to remove **{}** from your listing?", product_name).as_str() ).await?;
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

            send_hidden_msg(ctx, format!("
                Removed **{}** from your product page.\n
                Price: **{}** {}", 
                deleted.product_name.unwrap(),
                deleted.product_price,
                product_description,
            ).as_str())
            .await?; 
        }
        None => { send_hidden_msg(ctx, format!("**{}** doesn't exist in your product page.", product_name).as_str() ).await?; }
    }

    Ok(())
}


/// View a discord user product listing
#[poise::command(prefix_command, slash_command)]
pub async fn view_listing(
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
            send_hidden_msg(ctx, format!("**{}** product page.\n{:#?}", discord_user.mention(), user_list).as_str() ).await?; 
        }
        None => { send_hidden_msg(ctx, format!("**{}** doesn't have any listings or doesn't exist in database.", discord_user.mention()).as_str() ).await?; }

    }

    Ok(())
}

