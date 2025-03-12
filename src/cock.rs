use crate::server::get_database;
use mongodb::bson::{doc, Document};

pub async fn cock() -> mongodb::error::Result<()>{
    let database = get_database().await?;

    let _result: Option<Document> = database.collection("users").find_one(doc! {"sigma": "siggity"}).await?;

    Ok(())
}