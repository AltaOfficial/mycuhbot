use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;

#[allow(unused_imports)]
use crate::{Data, Error, Context};
use crate::{get_pool, update_db_new_user};

const GRAY_HIGHLIGHT: i32 = 0x747F8D;
const GREEN_HIGHLIGHT: i32 = 0x57F287;
const RED_HIGHLIGHT: i32 = 0xED4245;
//const EXPIRE_TIME: std::time::Duration = std::time::Duration::from_secs(300);
const EXPIRE_TIME: std::time::Duration = std::time::Duration::from_secs(15);

pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        balance(),
        transfer(),
        //request(),
        //add_product_to_listing(),
        //remove_product_in_listing(),
        //view_listing(),
    ]
}


// not sure if I'll use this function because I still need to make queries to get information but
// also need to check if user exist
#[allow(unused)]
async fn is_user_in(db: &sqlx::SqlitePool, discord_id: i64) -> bool {
    sqlx::query!("SELECT * FROM accounts WHERE discord_id = ?", discord_id)
        .fetch_one(db)
        .await
        .is_ok()
}


async fn send_hidden_msg(ctx: Context<'_>, msg: String) -> Result<poise::ReplyHandle<'_>, Error> {
    let reply = ctx.send(poise::CreateReply::default()
        .content(msg)
        .ephemeral(true)
    )
    .await?;

    Ok(reply)
}



// TODO, this function is a bit ugly might want to be cleaned up
/// Show bank balance
#[poise::command(prefix_command, slash_command)]
pub async fn balance(ctx: Context<'_>) -> Result<(), Error> {
    let requester: i64 = ctx.author().id.get() as i64;
    let pool = get_pool();

    let record = sqlx::query!("SELECT bucks FROM accounts WHERE discord_id = ?", requester)
        .fetch_optional(pool)
        .await?;

    //let record = is_user_in(pool, requester).await;

    // helper function that abstracts message being sent if user does or doesn't exist
    async fn send_balance_message(ctx: poise::Context<'_, Data, Error>, bal: i64, requester: i64) -> Result<(), Error> {
        println!("queryied bal from {} with balance value {}.", requester, bal);
        
        ctx.send(poise::CreateReply::default()
            .content(format!("You have **{}** Mycuh bucks", bal))
            .ephemeral(true)
        ).await?;

        Ok(())
    }

    /*
    if record {
        send_balance_message(ctx, record.bucks, requester).await?;
    } else {
        let record = update_db_new_user(ctx).await?;

        send_balance_message(ctx, record.2, requester).await?;
    };
    */

    match record {
        Some(record) => {
            send_balance_message(ctx, record.bucks, requester).await?;
        },

        None => {
            let record = update_db_new_user(ctx).await?;

            // I made the return type a tuple and its just so ugly
            send_balance_message(ctx, record.2, requester).await?;
        }
    };

    Ok(())
}


// this function is very large and has many duplicated parts of code, could be and prob should be cleaned up
/// Ask another member to transfer mycuh buck(s)
#[poise::command(prefix_command, slash_command)]
pub async fn transfer(
    ctx: Context<'_>,

    #[description = "Amount to transfer"]
    request_amount: u32,

    #[description = "Reason for transfer"]
    request_reason: String,

    #[description = "Member to transfer to/from"]
    recpient_user: serenity::User, 
) -> Result<(), Error> {
    let requester_user = ctx.author();
    let base_id = ctx.id().to_string();

    let pool = get_pool();

    // check if the user being requested is the same user
    /*
    if &recpient_user == requester_user {
        send_hidden_msg(ctx, "Cannot transfer Mycuh Bucks to yourself.".to_string()).await?;
        println!("User {} tried to transfer bucks to themselves", requester_user);

        return Ok(());
    }
    */

    let sender_id = requester_user.id.get() as i64;
    let recpient_id = recpient_user.id.get() as i64;

    /*
    let requested_user_bucks = sqlx::query!("
        SELECT bucks 
        FROM accounts 
        WHERE discord_id = ?", 
        recpient_id
    )
    .fetch_optional(pool)
    .await?;
    */

    // sender is assumed to be in database... might want to check to see if they acually exist TODO
    let bal = sqlx::query!(
        r#"
            SELECT a1.bucks AS sender_bal, a2.bucks AS recipient_bal
            FROM accounts as a1
            LEFT JOIN accounts as a2 ON a2.discord_id = ?
            WHERE a1.discord_id = ?;
        "#,
        recpient_id,
        sender_id
    )
    .fetch_one(pool)
    .await?; 
    println!("Sender: {} has bal: {},\nRecpient: {} has bal: {}", requester_user.tag(), bal.sender_bal, recpient_user.tag(), bal.recipient_bal);

    // check if user has requested user even has enough bucks 
    /*
    match requested_user_bucks {
        Some(request_user_bucks) => {
            if request_user_bucks.bucks >= request_amount as i64 {
                send_hidden_msg(ctx, format!("Sent transfer request to {} time till expire {}.", recpient_user.mention(), EXPIRE_TIME.as_secs()) ).await?;

            } else {
                //send_hidden_msg(ctx, format!("User {} only has {} bucks.", recpient_user.mention(), bal.recipient_bal)).await?;
                println!("User {} does not have enough Mycuh bucks to give.", requester_user);

                //return Ok(());
            }
        },
        None => {
            send_hidden_msg(ctx, "User does not exist in database.".to_string()).await?;
            return Ok(());
        }
    }
    */

    // this assumes that the user exist in the database
    if bal.recipient_bal < request_amount as i64 {
        send_hidden_msg(ctx, format!("User {} only has {} bucks.", recpient_user.mention(), bal.recipient_bal)).await?;
        println!("User {} does not have enough Mycuh bucks to give.", requester_user.tag());

        return Ok(());
    }
    send_hidden_msg(ctx, format!("Sent transfer request to {} time till expire {}.", recpient_user.mention(), EXPIRE_TIME.as_secs()) ).await?;

    


    let components = {
        let btn_yes = serenity::CreateButton::new(format!("{base_id}:yes"))
            .style(serenity::ButtonStyle::Secondary)
            .label("Yes")
            .emoji('✅');

        let btn_no = serenity::CreateButton::new(format!("{base_id}:no"))
            .style(serenity::ButtonStyle::Secondary)
            .label("No")
            .emoji('❌');

        vec![serenity::CreateActionRow::Buttons(vec![btn_yes, btn_no,] )]
    };





    let embed = serenity::CreateEmbed::default()
        .title("Mycuh Bucks Transfer Request")
        .color(GRAY_HIGHLIGHT)
        .description(format!(
            "{} is requesting to add **{}** mycuh buck(s) from {} to their myccount\n\
            Reason: **{}**\n\
            User Response: **{} secs** ⏳",

            requester_user.mention(),
            request_amount,
            recpient_user.mention(),
            request_reason,
            EXPIRE_TIME.as_secs(),
        ));


    // recpient dm hook to send/edit messages
    let dm = match recpient_user.create_dm_channel(ctx.serenity_context()).await {
        Ok(dm) => dm,
        Err(_) => {
            ctx.send(
                poise::CreateReply::default()
                    .content("That user has DMs disabled.")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };


    //let reply_handle = ctx.send(reply).await?;
    let message_handle = dm.send_message(
        ctx.serenity_context(), 
        serenity::CreateMessage::new()
            .embed(embed)
            .components(components.clone())
    ).await?;

    // recpient emoji selection data [yes/no/timeout] 
    let interaction = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(recpient_user.id)
        .timeout(EXPIRE_TIME)
        .filter(move |mci| {
            mci.data.custom_id == format!("{base_id}:yes") || mci.data.custom_id == format!("{base_id}:no")
        }
    ).await;

    match interaction {
        Some(mci) => {
            let mut accepted = false; 
            let status; 

            match mci.data.custom_id.as_str() {
                id if id.ends_with(":yes") => {
                    // TODO: sql query
                    accepted = true;
                    status = "Accepted ✅";
                    println!("User {} Accepted ✅", recpient_user.tag());
                }

                id if id.ends_with(":no") => {
                    status = "Denied ❌";
                    println!("User {} Denied ❌", recpient_user.tag());
                }

                _ => unreachable!(),
            }

            let mut msg = mci.message.clone();

            msg.edit(
                ctx, 
                serenity::EditMessage::new()
                .embeds(vec![
                    serenity::CreateEmbed::new()
                        .title("Bucks Transfer Request")
                        .color(if accepted { GREEN_HIGHLIGHT } else { RED_HIGHLIGHT })
                        .description(format!(
                            "{} requested **{}** buck(s) from {}\n\
                            Reason: **{}**\n\
                            User Response: **{}**",
                            requester_user.mention(),
                            request_amount,
                            recpient_user.mention(),
                            request_reason,
                            status
                        ))
                ])
                .components(vec![])
            )
            .await?;
        },

        _ => {
            // timeout
            let mut msg = message_handle.to_owned();

            msg.edit(
                ctx, 
                serenity::EditMessage::new()
                .embeds(vec![
                    serenity::CreateEmbed::new()
                        .title("Bucks Transfer Request")
                        .color(GRAY_HIGHLIGHT)
                        .description(format!(
                            "{} requested **{}** buck(s) from {}\n\
                            Reason: **{}**\n\
                            User Response: **Expired ⏱**",
                            requester_user.mention(),
                            request_amount,
                            recpient_user.mention(),
                            request_reason,
                        ))
                ])
                .components(vec![]),
            )
            .await?;
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
#[allow(dead_code, unused_variables)]
#[poise::command(prefix_command, slash_command)]
pub async fn add_product_to_listing(
    ctx: Context<'_>,

    #[description = "Name Of Product"]
    product_name: String,

    #[description = "Description Of Product"]
    product_description: String,

    #[description = "Amount of bucks to request"]
    price: u32,
) -> Result<(), Error> {
    // sql query
    
    
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
    println!("Are you sure you want to remove **{}** from your listing?", product_name);
    let yes = true;

    if yes {
        // sql query
    }
    
    Ok(())
}


/// Add a new product to listing
#[allow(dead_code, unused_variables)]
#[poise::command(prefix_command, slash_command)]
pub async fn view_listing(
    ctx: Context<'_>,

    #[description = "Member to view listings from"]
    user: serenity::User, // change to reponse from emoji from user
) -> Result<(), Error> {
    // sql query
    
    
    Ok(())
}

