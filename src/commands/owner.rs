use crate::{Data, Error, Context};


pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        quit(),
        //save_db(),
    ]
}


/// Stop The Bot
#[poise::command(prefix_command, slash_command)]
pub async fn quit(ctx: Context<'_>) -> Result<(), Error> {
    todo!("need to impl quit command");
    let owner_id = 1231232; // should not be hardcoded but not sure what to do :/

    if ctx.author().id != owner_id {
        ctx.send(
            poise::CreateReply::default()
                .content("You are not the owner, reporting incident.")
                .ephemeral(true)
        ).await?;

        // dm, log, do something
        println!("ALERT USER {} TRIED TO SHUT OFF BOT", ctx.author().tag());
        return Ok(());
    }

    ctx.say("Shutting down the bot now!").await?;

    // shutdown all instances of the bot in each guild
    ctx.framework().shard_manager().shutdown_all().await;


    Ok(())
}


/// Save SQL Database
#[allow(unused)]
#[poise::command(prefix_command, slash_command)]
pub async fn save_db(ctx: Context<'_>) -> Result<(), Error> {
    todo!("should call sql statement to save db");
    Ok(())
}

