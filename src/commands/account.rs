use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;

#[allow(unused_imports)]
use crate::{Data, Error, Context};

const GRAY_HIGHLIGHT: i32 = 0x747F8D;
const GREEN_HIGHLIGHT: i32 = 0x57F287;
const RED_HIGHLIGHT: i32 = 0xED4245;
//const EXPIRE_TIME: std::time::Duration = std::time::Duration::from_secs(300);
const EXPIRE_TIME: std::time::Duration = std::time::Duration::from_secs(5);

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


/// Show bank balance
#[poise::command(prefix_command, slash_command)]
pub async fn balance(ctx: Context<'_>) -> Result<(), Error> {
    //let requester = ctx.author();

    // sql query

    let bal = 100;
    
    ctx.send(poise::CreateReply::default()
        .content(format!("You have **{}** Mycuh bucks", bal))
        .ephemeral(true)
    ).await?;

    Ok(())
}


/// Ask another member to transfer mycuh buck(s)
//#[allow(dead_code, unused_variables)]
#[poise::command(prefix_command, slash_command)]
pub async fn transfer(
    ctx: Context<'_>,

    #[description = "Amount to transfer"]
    request_amount: u32,

    #[description = "Reason for transfer"]
    request_reason: String,

    #[description = "Member to transfer to/from "]
    user: serenity::User, // change to reponse from emoji from user
) -> Result<(), Error> {
    let requester = ctx.author();
    let base_id = ctx.id().to_string();

    let reply = {
        let btn_yes = serenity::CreateButton::new(format!("{base_id}:yes"))
            .style(serenity::ButtonStyle::Secondary)
            .label("Yes")
            .emoji('✅');

        let btn_no = serenity::CreateButton::new(format!("{base_id}:no"))
            .style(serenity::ButtonStyle::Secondary)
            .label("No")
            .emoji('❌');

        let components = vec![serenity::CreateActionRow::Buttons(vec![
            btn_yes,
            btn_no,
        ])];

        poise::CreateReply::default()
            .embed(
                serenity::CreateEmbed::default()
                    .title("Mycuh Bucks Transfer Request")
                    .color(GRAY_HIGHLIGHT)
                    .description(format!(
                        "{} is requesting to add **{}** mycuh buck(s) from {} to their myccount\n\
                        Reason: **{}**\n\
                        User Response: **{}** ⏳",

                        requester.mention(),
                        request_amount,
                        user.mention(),
                        request_reason,
                        EXPIRE_TIME.as_secs(),
                    ))
            )
            .components(components)
    };

    let reply_handle = ctx.send(reply).await?;
    let message = reply_handle.message().await?;

    let interaction = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        //.message_id(reply.message().await?.id) // only this message
        //.message_id(reply.id)                  // only this message
        .author_id(user.id)                      // ONLY the requested user
        .timeout(EXPIRE_TIME)
        .filter(move |mci| {                     // not sure what mci stands for but filterout the correct id from discord
            mci.data.custom_id == format!("{base_id}:yes")
                || mci.data.custom_id == format!("{base_id}:no")
        })
        .await;

    match interaction {
        Some(mci) => {
            let mut accepted = false; 
            let status; 

            match mci.data.custom_id.as_str() {
                id if id.ends_with(":yes") => {
                    // TODO: sql query
                    accepted = true;
                    status = "Accepted ✅";
                    println!("Accepted ✅");
                }

                id if id.ends_with(":no") => {
                    status = "Denied ❌";
                    println!("Denied ❌");
                }

                _ => unreachable!("filtered custom_id should prevent this"),
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
                            requester.mention(),
                            request_amount,
                            user.mention(),
                            request_reason,
                            status
                        ))
                ])
                .components(vec![]),
            )
            .await?;
        },

        _ => {
            // timeout
            let mut msg = message.into_owned();

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
                            requester.mention(),
                            request_amount,
                            user.mention(),
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
pub async fn request(
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

