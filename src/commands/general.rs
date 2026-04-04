use crate::utils::{send_msg, edit_msg};
use crate::{Data, Error, Context};


pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        help(),
        ping(),
        latency(),
    ]
}

/// List all help commands
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    //#[rest] // IMPORTANT for prefix commands
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "Hello!\n\
If you want more information about a specific command, try:\n\
  `/help help`\n",
        show_context_menu_commands: true,
        show_subcommands: true,

        ..Default::default()
    };

    poise::builtins::help(
        ctx,
        command.as_deref(),
        config,
    )
    .await?;

    Ok(())
}

/// Pong? 
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    send_msg(ctx, "Pong!", true).await?;
    println!("User: {} Pinged!", ctx.author().id);

    Ok(())
}

/// Gets the latency (lag time) of the bot
#[poise::command(prefix_command, slash_command, aliases("lag"))]
pub async fn latency(ctx: Context<'_>) -> Result<(), Error> {
    use std::time::Instant;

    let start_time = Instant::now();
    let handle = send_msg(ctx, "Calculating ping...", true).await?;
    let end_time = Instant::now();

    let calc = (end_time - start_time).as_millis();

    edit_msg(ctx, &handle, format!("{}ms", calc).as_str() ).await?;
    println!("Lag: {}ms", calc);

    Ok(())
}

