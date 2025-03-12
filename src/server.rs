use std::env;
use dotenv::dotenv;

use serde_json::to_string_pretty;
use mongodb::{ bson::*, options::{ ClientOptions, ServerApi, ServerApiVersion }, Client, Collection };

#[tokio::main]
pub async fn main() -> mongodb::error::Result<Client> {
    dotenv().ok();
    let url = env::var("database_string").unwrap();
    let mut client_options = ClientOptions::parse(url).await?;

    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    // Create a new client and connect to the server
    let client = Client::with_options(client_options)?;

    // list db
    /*
    let db_list = client.list_database_names().await?;
    println!("{:?}", db_list);
    */

    let db = client.database("test");

    let collections: Collection<Document> = db.collection("users");

    //let my_coll = client.database("test").collection::<Document>("users"); // short hand for
    //above

    //let result = collections.insert_one(doc! {"name":"Jack", "age":30}, None).await?;

    let filter = doc! {"username": "mr_black_118"};
    if let Some(document) = collections.find_one(filter).await? {
        println!("User: {}", to_string_pretty(&document).unwrap());
    } else {
        println!("could not find document");
    }

    Ok(client)
}

