use std::fmt::Write;

use anyhow::{bail, Context, Result};
use heck::{CamelCase, SnakeCase};

use crate::{
    options::Options,
    schema::{Column, SqlType, Table},
};

fn generate_prelude(dst: &mut impl Write, options: &Options) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");

    writeln!(dst, "// GENERATED CODE (ch2rs v{})", version)?;
    writeln!(dst, "#![cfg_attr(rustfmt, rustfmt::skip)]")?;
    writeln!(dst, "#![allow(warnings)]")?;
    writeln!(dst, "#![allow(clippy::all)]")?;
    writeln!(dst, "\n// Generated with the following options:")?;
    writeln!(dst, "/*\n{}\n*/", options.format().replace('\n', "\n    "))?;

    Ok(())
}

fn generate_row(dst: &mut impl Write, table: &Table, options: &Options) -> Result<()> {
    writeln!(dst, "#[derive(Debug, clickhouse::Row)]")?;

    if options.serialize {
        writeln!(dst, "#[derive(serde::Serialize)]")?;
    }

    if options.deserialize {
        writeln!(dst, "#[derive(serde::Deserialize)]")?;
    }

    let mut buffer = String::new();

    for column in &table.columns {
        generate_field(&mut buffer, column, options)
            .with_context(|| format!("failed to generate the `{}` field", column.name))?;
    }

    let has_lifetime = buffer.contains("'a");
    if has_lifetime {
        writeln!(dst, "pub struct Row<'a> {{")?;
    } else {
        writeln!(dst, "pub struct Row {{")?;
    }

    dst.write_str(&buffer)?;
    writeln!(dst, "}}")?;
    Ok(())
}

fn generate_field(dst: &mut impl Write, column: &Column, options: &Options) -> Result<()> {
    if let Some(attr) = make_attribute(column, options) {
        writeln!(dst, "{}", attr)?;
    }

    let name = column.name.to_snake_case();
    let type_ = make_type(column, options)?;

    writeln!(dst, "    pub {}: {},", name, type_)?;
    Ok(())
}

fn make_attribute(column: &Column, options: &Options) -> Option<String> {
    if options.bytes.iter().any(|b| b == &column.name) {
        return Some(r#"    #[serde(with = "serde_bytes")]"#.into());
    }

    // Add nothing if the column is overrided by name or type.
    if find_override(&column.name, &column.type_, options).is_some() {
        return None;
    }

    if column.type_ == SqlType::UUID {
        return Some(r#"    #[serde(with = "::clickhouse::serde::uuid")]"#.into());
    }

    None
}

fn make_type(column: &Column, options: &Options) -> Result<String> {
    do_make_type(&column.name, &column.type_, options)
}

fn do_make_type(name: &str, sql_type: &SqlType, options: &Options) -> Result<String> {
    if let Some(type_) = find_override(name, sql_type, options) {
        return Ok(type_.into());
    }

    Ok(match sql_type {
        SqlType::UInt8 => "u8".into(),
        SqlType::UInt16 => "u16".into(),
        SqlType::UInt32 => "u32".into(),
        SqlType::UInt64 => "u64".into(),
        SqlType::Int8 => "i8".into(),
        SqlType::Int16 => "i16".into(),
        SqlType::Int32 => "i32".into(),
        SqlType::Int64 => "i64".into(),
        SqlType::Bool => "bool".into(),
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
        SqlType::UUID => "::uuid::Uuid".into(),
        //SqlType::Decimal(_prec, _scale) => todo!(),
        SqlType::Enum8(_) | SqlType::Enum16(_) => name.to_camel_case(),
        SqlType::Array(inner) => format!("Vec<{}>", do_make_type(name, inner, options)?),
        SqlType::Tuple(inner) => {
            let inner = inner
                .iter()
                .map(|i| do_make_type(name, i, options).map(|t| format!("{}, ", t)))
                .collect::<Result<String>>()?;

            format!("({})", inner)
        }
        SqlType::Map(key, value) => {
            let tup = Box::new(SqlType::Tuple(vec![(**key).clone(), (**value).clone()]));
            do_make_type(name, &SqlType::Array(tup), options)?
        }
        SqlType::Nullable(inner) => format!("Option<{}>", do_make_type(name, inner, options)?),
        _ => bail!(
            "there is no default impl for {}, use -T or -O to specify it",
            sql_type
        ),
    })
}

fn find_override<'a>(name: &str, sql_type: &SqlType, options: &'a Options) -> Option<&'a str> {
    // Find override by a column's name.
    if let Some(o) = options.overrides.iter().find(|o| o.column == name) {
        return Some(&o.type_);
    }

    // Find override by SQL type.
    if let Some(t) = options.types.iter().find(|t| &t.sql == sql_type) {
        return Some(&t.type_);
    }

    None
}

fn generate_enums(dst: &mut impl Write, table: &Table, options: &Options) -> Result<()> {
    fn find_enum(t: &SqlType) -> Option<(bool, &[(String, i32)])> {
        match t {
            SqlType::Enum8(v) => Some((false, v)),
            SqlType::Enum16(v) => Some((true, v)),
            SqlType::Array(inner) => find_enum(inner),
            SqlType::Tuple(inner) => inner.iter().flat_map(find_enum).next(),
            SqlType::Nullable(inner) => find_enum(inner),
            _ => None,
        }
    }

    for column in &table.columns {
        if let Some((is_extended, variants)) = find_enum(&column.type_) {
            generate_enum(
                dst,
                &column.name.to_camel_case(),
                is_extended,
                variants,
                options,
            )?;
            writeln!(dst)?;
        }
    }

    Ok(())
}

fn generate_enum(
    dst: &mut impl Write,
    name: &str,
    is_extended: bool,
    variants: &[(String, i32)],
    options: &Options,
) -> Result<()> {
    writeln!(dst, "#[derive(Debug)]")?;

    if options.serialize {
        writeln!(dst, "#[derive(serde_repr::Serialize_repr)]")?;
    }

    if options.deserialize {
        writeln!(dst, "#[derive(serde_repr::Deserialize_repr)]")?;
    }

    if is_extended {
        writeln!(dst, "#[repr(i16)]")?;
    } else {
        writeln!(dst, "#[repr(i8)]")?;
    }

    writeln!(dst, "pub enum {} {{", name)?;

    for (name, value) in variants {
        writeln!(dst, "    {} = {},", prepare_name_ident(name), value)?;
    }

    writeln!(dst, "}}")?;

    Ok(())
}

fn prepare_name_ident(name: &str) -> String {
    if name.trim().is_empty() {
        "Empty".into()
    } else {
        name.to_camel_case()
    }
}

pub fn generate(table: &Table, options: &Options) -> Result<String> {
    let mut code = String::new();
    generate_prelude(&mut code, options).context("failed to generate a prelude")?;
    writeln!(code)?;
    generate_row(&mut code, table, options).context("failed to generate a row")?;
    writeln!(code)?;
    generate_enums(&mut code, table, options).context("failed to generate enums")?;
    Ok(code.trim().to_string())
}
