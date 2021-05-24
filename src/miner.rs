use anyhow::{bail, Context, Result};
use clickhouse::{Client, Reflection};
use serde::Deserialize;

use crate::{
    options::Options,
    schema::{Column, SqlType, Table},
};

fn make_client(options: &Options) -> Client {
    let url = if !options.url.starts_with("http") {
        format!("http://{}", options.url)
    } else {
        options.url.clone()
    };

    let mut client = Client::default().with_url(&url);

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
    name: String,
    #[serde(rename = "type")]
    type_: String,
    comment: String,
}

async fn fetch_raw_columns(client: &Client, options: &Options) -> Result<Vec<RawColumn>> {
    Ok(client
        .query(
            "
            SELECT ?fields
              FROM system.columns
             WHERE database = ?
               AND table = ?
        ",
        )
        .bind(&options.database)
        .bind(&options.table)
        .fetch_all::<RawColumn>()
        .await?)
}

fn make_table(raw_columns: Vec<RawColumn>, options: &Options) -> Result<Table> {
    let mut columns = Vec::new();

    for raw_column in raw_columns {
        let reason = format!("failed to handle the `{}` column", raw_column.name);
        let column = make_column(raw_column).context(reason)?;
        columns.push(column);
    }

    Ok(Table {
        database: options.database.clone(),
        name: options.table.clone(),
        columns,
    })
}

fn make_column(raw: RawColumn) -> Result<Column> {
    let type_ = parse_type(&raw.type_)
        .with_context(|| format!("failed to parse the `{}` type", raw.type_))?;

    Ok(Column {
        name: raw.name,
        type_,
        comment: raw.comment,
    })
}

pub fn parse_type(raw: &str) -> Result<SqlType> {
    let raw = extract_inner(raw, "LowCardinality").unwrap_or(raw);

    // TODO: unwrap `SimpleAggregateFunction`.

    Ok(match raw {
        "UInt8" => SqlType::UInt8,
        "UInt16" => SqlType::UInt16,
        "UInt32" => SqlType::UInt32,
        "UInt64" => SqlType::UInt64,
        "Int8" => SqlType::Int8,
        "Int16" => SqlType::Int16,
        "Int32" => SqlType::Int32,
        "Int64" => SqlType::Int64,
        "String" => SqlType::String,
        "Float32" => SqlType::Float32,
        "Float64" => SqlType::Float64,
        "Date" => SqlType::Date,
        "DateTime" => SqlType::DateTime(None),
        "IPv4" => SqlType::Ipv4,
        "IPv6" => SqlType::Ipv6,
        "UUID" => SqlType::Uuid,
        _ => {
            // Nullable(type)
            if let Some(inner) = extract_inner(raw, "Nullable") {
                SqlType::Nullable(Box::new(parse_type(inner)?))
            }
            // DateTime(tz)
            else if let Some(inner) = extract_inner(raw, "DateTime") {
                SqlType::DateTime(Some(inner.into()))
            }
            // DateTime64(prec)
            // DateTime64(prec, tz)
            else if let Some(inner) = extract_inner(raw, "DateTime64") {
                let (prec, tz) = inner
                    .split_once(", ")
                    .map_or((inner, None), |(p, tz)| (p, Some(tz)));
                let prec = prec.parse().context("invalid precision")?;
                SqlType::DateTime64(prec, tz.map(Into::into))
            }
            // Enum8('K' = v, 'K2' = v2)
            else if let Some(inner) = extract_inner(raw, "Enum8") {
                SqlType::Enum8(parse_kv_list(inner).context("invalid enum")?)
            }
            // Enum16('K' = v, 'K2' = v2)
            else if let Some(inner) = extract_inner(raw, "Enum16") {
                SqlType::Enum16(parse_kv_list(inner).context("invalid enum")?)
            }
            // Decimal(prec, scale)
            else if let Some(inner) = extract_inner(raw, "Decimal") {
                let (prec, scale) = inner.split_once(", ").context("invalid decimal")?;
                let prec = prec.parse().context("invalid prec")?;
                let scale = scale.parse().context("invalid precision")?;
                SqlType::Decimal(prec, scale)
            }
            // FixedString(size)
            else if let Some(inner) = extract_inner(raw, "FixedString") {
                SqlType::FixedString(inner.parse().context("invalid size")?)
            }
            // Array(type)
            else if let Some(inner) = extract_inner(raw, "Array") {
                SqlType::Array(Box::new(parse_type(inner)?))
            }
            // Tuple(a, b)
            else if let Some(inner) = extract_inner(raw, "Tuple") {
                SqlType::Tuple(
                    inner
                        .split(", ")
                        .map(|t| parse_type(t))
                        .collect::<Result<Vec<_>>>()?,
                )
            }
            // Map(key, value)
            else if let Some(inner) = extract_inner(raw, "Map") {
                let (key, value) = inner.split_once(", ").context("invalid map")?;
                let key = parse_type(key).context("invalid key")?;
                let value = parse_type(value).context("invalid value")?;
                SqlType::Map(Box::new(key), Box::new(value))
            } else {
                bail!("unknown type");
            }
        }
    })
}

fn extract_inner<'a>(raw: &'a str, wrapper: &str) -> Option<&'a str> {
    if raw.starts_with(wrapper) && raw[wrapper.len()..].starts_with('(') {
        Some(&raw[wrapper.len() + 1..raw.len() - 1])
    } else {
        None
    }
}

// 'K' = v, 'K2' = v2
fn parse_kv_list(raw: &str) -> Result<Vec<(String, i32)>> {
    raw.split(", ")
        .map(|pair| {
            let (k, v) = pair
                .split_once(" = ")
                .with_context(|| format!("invalid key-value pair `{}`", pair))?;
            let k = k.get(1..k.len() - 1).context("invalid variant key")?;
            let v = v.parse().context("invalid variant value")?;
            Ok((k.into(), v))
        })
        .collect()
}

pub async fn mine(options: &Options) -> Result<Table> {
    let client = make_client(options);
    let raw_columns = fetch_raw_columns(&client, options)
        .await
        .context("failed to fetch columns")?;
    let table = make_table(raw_columns, options).context("failed to make the table")?;
    Ok(table)
}
