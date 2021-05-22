use anyhow::{Context, Result};
use clickhouse::{Client, Reflection};
use serde::Deserialize;

use crate::{
    options::Options,
    schema::{Column, Schema, SqlType, Table},
};

fn make_client(options: &Options) -> Client {
    let mut client = Client::default().with_url(&options.url);

    if let Some(user) = &options.user {
        client = client.with_user(user);
    }

    if let Some(password) = &options.password {
        client = client.with_password(password);
    }

    client
}

#[derive(Debug, Deserialize, Reflection)]
struct RawColumn {
    database: String,
    table: String,
    name: String,
    #[serde(rename = "type")]
    type_: String,
    comment: String,
}

async fn fetch_raw_columns(client: &Client) -> Result<Vec<RawColumn>> {
    Ok(client
        .query("SELECT ?fields FROM system.columns")
        .fetch_all::<RawColumn>()
        .await?)
}

pub async fn mine(options: &Options) -> Result<Schema> {
    let client = make_client(options);
    let raw_columns = fetch_raw_columns(&client)
        .await
        .context("cannot fetch columns")?;

    for c in raw_columns {
        println!("{:?}", c);
    }
    todo!();
}
