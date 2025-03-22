use serenity::all::{ Result, Error, Client};

pub mod stock_market;
mod server;
mod utilities;
mod register_commands;

//use mycuhbot::*;

#[tokio::main]
async fn main() -> Result<(), Error>{
    let mut client: Client = utilities::server_client::get_client().await?; // this creates the discord bot client
    if let Err(why) = client.start().await { // this starts the discord bot
        // and if an error occurs during runtime
        println!("Error with client: {:?}", why)
    }
    register_commands::register_commands().await?; // TODO: Register chat commands (not yet converted from Discord.js)
    Ok(())
    // mongodb server call
    //let server = server::main();
    //println!("{:?}", server.unwrap());
}

