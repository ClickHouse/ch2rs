use anyhow::{bail, Context, Result};
use heck::{CamelCase, SnakeCase};

use crate::{
    options::Options,
    schema::{Column, SqlType, Table},
};

fn generate_prelude(options: &Options) -> Result<()> {
    println!("// GENERATED CODE");
    println!("/* generated with the following options:");
    println!("\n{}\n", options.format());
    println!("*/\n");

    // TODO: print options.

    Ok(())
}

fn generate_row(table: &Table, options: &Options) -> Result<()> {
    println!("#[derive(Debug, serde::Serialize, clickhouse::Reflection)]");

    if options.owned {
        println!("pub struct OwnedRow {{");
    } else {
        println!("pub struct BorrowedRow<'a> {{");
    }

    for column in &table.columns {
        generate_field(column, options)
            .with_context(|| format!("failed to generate the `{}` field", column.name))?;
    }

    println!("}}");

    Ok(())
}

fn generate_field(column: &Column, options: &Options) -> Result<()> {
    let name = column.name.to_snake_case();
    let type_ = make_type(&column.type_, &column.name, options)?;
    println!("    pub {}: {},", name, type_);
    Ok(())
}

fn make_type(raw: &SqlType, name: &str, options: &Options) -> Result<String> {
    if let Some(o) = options.overrides.iter().find(|o| o.column == name) {
        return Ok(o.type_.clone());
    }

    if let Some(t) = options.types.iter().find(|t| &t.sql == raw) {
        return Ok(t.type_.clone());
    }

    // TODO: custom types.
    Ok(match raw {
        SqlType::UInt8 => "u8".into(),
        SqlType::UInt16 => "u16".into(),
        SqlType::UInt32 => "u32".into(),
        SqlType::UInt64 => "u64".into(),
        SqlType::Int8 => "i8".into(),
        SqlType::Int16 => "i16".into(),
        SqlType::Int32 => "i32".into(),
        SqlType::Int64 => "i64".into(),
        SqlType::String if options.owned => "String".into(),
        SqlType::String => "&'a str".into(),
        //SqlType::FixedString(size) => todo!(),
        SqlType::Float32 => "f32".into(),
        SqlType::Float64 => "f64".into(),
        //SqlType::Date => todo!(),
        //SqlType::DateTime(_) => todo!(),
        //SqlType::DateTime64(_, _) => todo!(),
        //SqlType::Ipv4 => todo!(),
        //SqlType::Ipv6 => todo!(),
        //SqlType::Uuid => todo!(),
        //SqlType::Decimal(_prec, _scale) => todo!(),
        SqlType::Enum8(_) | SqlType::Enum16(_) => name.to_camel_case(),
        SqlType::Array(inner) => format!("Vec<{}>", make_type(inner, name, options)?),
        SqlType::Tuple(inner) => inner
            .iter()
            .map(|i| make_type(i, name, options).map(|t| format!("{}, ", t)))
            .collect::<Result<_>>()?,
        //SqlType::Map(_key, _value) => todo!(),
        SqlType::Nullable(inner) => format!("Option<{}>", make_type(inner, name, options)?),
        _ => bail!(
            "there is no default impl for {:?}, use -T or -O to specify it",
            raw
        ),
    })
}

pub fn generate(table: &Table, options: &Options) -> Result<()> {
    generate_prelude(options).context("failed to generate a prelude")?;
    generate_row(table, options).context("failed to generate a row")?;

    Ok(())
}
