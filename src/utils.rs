use poise::serenity_prelude as serenity;
use crate::{Error, Context};

pub const USER_OWNER: serenity::UserId = serenity::UserId::new(329041881734578186); // your Discord ID
pub const GRAY_HIGHLIGHT: i32 = 0x747F8D;
pub const GREEN_HIGHLIGHT: i32 = 0x57F287;
pub const RED_HIGHLIGHT: i32 = 0xED4245;
pub const BLUE_HIGHLIGHT: i32 = 0x00EEEE;

pub const EXPIRE_TIME: std::time::Duration = std::time::Duration::from_secs(300);


/// Sends a message 'msg' to the user from 'ctx'
pub async fn send_msg<'a>(ctx: Context<'a>, msg: &str, hidden: bool) -> Result<poise::ReplyHandle<'a>, Error> {
    let reply = ctx.send(poise::CreateReply::default()
        .content(msg)
        .ephemeral(hidden)
    )
    .await?;

    Ok(reply)
}

pub async fn send_embed<'a> (
    ctx: Context<'a>,
    title: &str,
    description: &str,
    color: i32,
) -> Result<poise::ReplyHandle<'a>, Error> {
    let reply = ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title(title)
                .description(description)
                .color(color)
        )
    )
    .await?;

    Ok(reply)
}

pub async fn create_embed<'a> (
    //ctx: Context<'a>,
    title: &str,
    description: &str,
    color: i32,
) -> Result<serenity::CreateEmbed, Error> {
    let embed = serenity::CreateEmbed::default()
        .title(title)
        .description(description)
        .color(color);

    Ok(embed)
}

#[allow(unused)]
pub async fn send_dm_embed<'a> (
    ctx: Context<'a>,
    title: &str,
    description: &str,
    color: i32,
) -> Result<poise::ReplyHandle<'a>, Error> {
    let reply = ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title(title)
                .description(description)
                .color(color)
        )
    )
    .await?;

    Ok(reply)
}

pub async fn edit_msg<'a>(
    ctx: Context<'a>,
    reply: &poise::ReplyHandle<'a>,
    new_content: &str,
) -> Result<(), Error> {
    reply.edit(ctx, poise::CreateReply::default().content(new_content)).await?;

    Ok(())
}




/// user button choice component help function
pub fn emoji_buttons_components(base_id: &str) -> Vec<serenity::CreateActionRow> {
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

pub async fn send_confirmation_msg(
    ctx: Context<'_>,
    user: &serenity::User,
    embed: serenity::CreateEmbed,
    //base_id: &str,
) -> Result<serenity::Message, serenity::Error> {
    // this is retarded, might be a better rust way of doing this but goes from u64 -> String -> &str
    let base_id = ctx.id().to_string();
    let base_id = base_id.as_str();

    let dm = /*match*/ user.create_dm_channel(ctx.serenity_context()).await?; /*{
        Ok(dm) => { dm },
        Err(_) => {
            //send_msg(ctx, "That user has DMs disabled.", true).await?;
            return Ok(());
        }
    };
    */

    dm.send_message(
        ctx,
        serenity::CreateMessage::new()
            .embed(embed)
            .components(emoji_buttons_components(base_id))
    )
    .await
}

/*
pub async fn send_confirmation_msg(
    ctx: Context<'_>,
    channel: &serenity::ChannelId,
    embed: serenity::CreateEmbed,
    base_id: &String,
) -> Result<serenity::Message, serenity::Error> {
    let dm = match recipient_user.create_dm_channel(ctx.serenity_context()).await {
        Ok(dm) => dm,
        Err(_) => {
            send_msg(ctx, "That user has DMs disabled.", true).await?;
            return Ok(());
        }
    };
    channel
        .send_message(
            ctx,
            serenity::CreateMessage::new()
                .embed(embed)
                .components(emoji_buttons_components(&base_id))
        )
        .await
}
*/


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


