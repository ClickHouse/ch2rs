use std::fmt::Write;

use anyhow::{Context, Result};
use structopt::StructOpt;

use crate::schema::SqlType;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(short = "U", default_value = "localhost:8123")]
    pub url: String,
    #[structopt(short = "u")]
    pub user: Option<String>,
    #[structopt(short = "p")]
    pub password: Option<String>,

    #[structopt(short = "d", default_value = "default")]
    pub database: String,
    pub table: String,

    #[structopt(short = "S")]
    pub serialize: bool,
    #[structopt(short = "D")]
    pub deserialize: bool,
    #[structopt(long)]
    pub owned: bool,
    #[structopt(short = "T", parse(try_from_str = parse_type), number_of_values = 1)]
    pub types: Vec<Type>,
    #[structopt(short = "O", parse(try_from_str = parse_override), number_of_values = 1)]
    pub overrides: Vec<Override>,
}

#[derive(Debug)]
pub struct Type {
    pub sql: SqlType,
    pub type_: String,
}

fn parse_type(s: &str) -> Result<Type> {
    let (sql, type_) = s.split_once('=').context("invalid key-value")?;
    Ok(Type {
        sql: crate::miner::parse_type(&sql)?,
        type_: type_.into(),
    })
}

#[derive(Debug)]
pub struct Override {
    pub column: String,
    pub type_: String,
}

fn parse_override(s: &str) -> Result<Override> {
    let (column, type_) = s.split_once('=').context("invalid key-value")?;
    Ok(Override {
        column: column.into(),
        type_: type_.into(),
    })
}

impl Options {
    pub fn format(&self) -> String {
        let mut s = String::new();

        let _ = write!(&mut s, "ch2rs {}", self.table);

        if self.database != "default" {
            let _ = write!(&mut s, " -d {}", self.database);
        }

        if self.serialize {
            s.push_str(" -S");
        }

        if self.deserialize {
            s.push_str(" -D");
        }

        if self.owned {
            s.push_str(" --owned");
        }

        s.push_str(" \\\n");

        for t in &self.types {
            let _ = writeln!(&mut s, "    -T '{}={}' \\", t.sql, t.type_);
        }

        for o in &self.overrides {
            let _ = writeln!(&mut s, "    -O '{}={}' \\", o.column, o.type_);
        }

        s.trim_end_matches(|c| c == '\\' || c == ' ' || c == '\n')
            .into()
    }
}
