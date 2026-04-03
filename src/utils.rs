use poise::serenity_prelude as serenity;
use crate::{Error, Context};

/// Sends a hidden message 'msg' to the user from 'ctx' 
pub async fn send_hidden_msg<'a>(ctx: Context<'a>, msg: &str) -> Result<poise::ReplyHandle<'a>, Error> {
    let reply = ctx.send(poise::CreateReply::default()
        .content(msg)
        .ephemeral(true)
    )
    .await?;

    Ok(reply)
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

/// user button choice component help function
pub fn emoji_buttons_components(base_id: &String) -> Vec<serenity::CreateActionRow> {
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


