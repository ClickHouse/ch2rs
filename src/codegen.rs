use anyhow::{Context, Result};
use heck::CamelCase;

use crate::{
    options::Options,
    schema::{Column, SqlType, Table},
};

fn generate_prelude(options: &Options) -> Result<()> {
    println!("// GENERATED CODE");

    // TODO: print options.

    Ok(())
}

fn generate_row(table: &Table, options: &Options, owned: bool) -> Result<()> {
    if owned {
        println!("pub struct OwnedRow {{");
    } else {
        println!("pub struct BorrowedRow<'a> {{");
    }

    for column in &table.columns {
        generate_field(column, options, owned)
            .with_context(|| format!("failed to generate the `{}` field", column.name))?;
    }

    Ok(())
}

fn generate_field(column: &Column, options: &Options, owned: bool) -> Result<()> {
    let type_ = make_type(&column.type_, &column.name, options, owned);
    println!("    pub {}: {},", column.name, type_);
    Ok(())
}

fn make_type(raw: &SqlType, name: &str, options: &Options, owned: bool) -> String {
    // TODO: custom types.
    match raw {
        SqlType::UInt8 => "u8".into(),
        SqlType::UInt16 => "u16".into(),
        SqlType::UInt32 => "u32".into(),
        SqlType::UInt64 => "u64".into(),
        SqlType::Int8 => "i8".into(),
        SqlType::Int16 => "i16".into(),
        SqlType::Int32 => "i32".into(),
        SqlType::Int64 => "i64".into(),
        SqlType::String if owned => "String".into(),
        SqlType::String => "&'a str".into(),
        SqlType::FixedString(_size) => todo!(),
        SqlType::Float32 => "f32".into(),
        SqlType::Float64 => "f64".into(),
        SqlType::Date => todo!(),
        SqlType::DateTime(_) => todo!(),
        SqlType::DateTime64(_, _) => "std::time::SystemTime".into(),
        SqlType::Ipv4 => todo!(),
        SqlType::Ipv6 => todo!(),
        SqlType::Uuid => todo!(),
        SqlType::Decimal(_prec, _scale) => todo!(),
        SqlType::Enum8(_) | SqlType::Enum16(_) => name.to_camel_case(),
        SqlType::Array(inner) => format!("Vec<{}>", make_type(inner, name, options, owned)),
        SqlType::Tuple(_inner) => "kek".into(),
        SqlType::Map(_key, _value) => todo!(),
        SqlType::Nullable(inner) => format!("Option<{}>", make_type(inner, name, options, owned)),
    }
}

pub fn generate(table: &Table, options: &Options) -> Result<()> {
    generate_prelude(options).context("failed to generate a prelude")?;

    // TODO !!!: Focus on one table only.

    generate_row(table, options, true).with_context(|| {
        format!(
            "failed to generate a owned row for the `{}` table",
            table.name
        )
    })?;

    generate_row(table, options, false).with_context(|| {
        format!(
            "failed to generate a borrowed row for the `{}` table",
            table.name
        )
    })?;

    Ok(())
}
