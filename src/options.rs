use std::fmt::Write;

use anyhow::{Context, Result};
use structopt::StructOpt;

use crate::schema::SqlType;

#[derive(Debug, StructOpt)]
pub struct Options {
    /// ClickHouse server's URL.
    #[structopt(short = "U", default_value = "localhost:8123")]
    pub url: String,
    #[structopt(short = "u")]
    pub user: Option<String>,
    #[structopt(short = "p")]
    pub password: Option<String>,

    /// A database where the table is placed in.
    #[structopt(short = "d", default_value = "default")]
    pub database: String,
    /// The table's name.
    pub table: String,

    /// Generate `Serialize` instances.
    #[structopt(short = "S")]
    pub serialize: bool,
    /// Generate `Deserialize` instances.
    #[structopt(short = "D")]
    pub deserialize: bool,
    /// Generate only owned types.
    #[structopt(long)]
    pub owned: bool,
    /// Override the type, e.g. 'Decimal(18, 9)=fixnum::FixedPoint<i64, typenum::U9>'
    #[structopt(short = "T", parse(try_from_str = parse_type), number_of_values = 1)]
    pub types: Vec<Type>,
    /// Override the type of the provided column.
    #[structopt(short = "O", parse(try_from_str = parse_override), number_of_values = 1)]
    pub overrides: Vec<Override>,
    /// Add `#[serde(with = "serde_bytes")]` to the provided column.
    #[structopt(short = "B")]
    pub bytes: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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

        // -T
        let mut types = self.types.iter().collect::<Vec<_>>();
        types.sort();

        for t in types {
            let _ = writeln!(&mut s, "    -T '{}={}' \\", t.sql, t.type_);
        }

        // -O
        let mut overrides = self.overrides.iter().collect::<Vec<_>>();
        overrides.sort();

        for o in overrides {
            let _ = writeln!(&mut s, "    -O '{}={}' \\", o.column, o.type_);
        }

        // -B
        let mut bytes = self.bytes.iter().collect::<Vec<_>>();
        bytes.sort();

        for b in bytes {
            let _ = writeln!(&mut s, "    -B '{}' \\", b);
        }

        s.trim_end_matches(|c| c == '\\' || c == ' ' || c == '\n')
            .into()
    }
}
