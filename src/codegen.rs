use std::fmt::Write;

use anyhow::{bail, Context, Result};
use heck::{ToSnakeCase, ToUpperCamelCase};

use crate::{
    options::{Options, Temporal},
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

    for derive in &options.derives {
        writeln!(dst, "#[derive({})]", derive)?;
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
    if let Some(attr) = make_attribute(column, options)? {
        writeln!(dst, "{}", attr)?;
    }

    let name = column.name.to_snake_case();
    let type_ = make_type(column, options)?;

    for comment_line in column.comment.lines() {
        writeln!(dst, "    /// {}", comment_line)?;
    }

    writeln!(dst, "    pub {}: {},", name, type_)?;
    Ok(())
}

fn is_sentinel(column_name: &str, options: &Options) -> bool {
    options.sentinels.iter().any(|s| s == column_name)
}

fn sentinel_mod_name(column_name: &str) -> String {
    format!("sentinel_{}", column_name.to_snake_case())
}

fn make_attribute(column: &Column, options: &Options) -> Result<Option<String>> {
    if options.bytes.iter().any(|b| b == &column.name) {
        // Works also for `Option<_>`.
        return Ok(Some(r#"    #[serde(with = "serde_bytes")]"#.into()));
    }

    if is_sentinel(&column.name, options) {
        let mod_name = sentinel_mod_name(&column.name);
        return Ok(Some(format!(r#"    #[serde(with = "{}")]"#, mod_name)));
    }

    // Add nothing if the column is overrided by name or type.
    if find_override(&column.name, &column.type_, options).is_some() {
        return Ok(None);
    }

    let (inner, is_option) = match &column.type_ {
        SqlType::Nullable(inner) => (inner.as_ref(), true),
        _ => (&column.type_, false),
    };

    let base = match inner {
        SqlType::UUID => Ok(Some("::clickhouse::serde::uuid".into())),
        SqlType::IPv4 => Ok(Some("::clickhouse::serde::ipv4".into())),
        SqlType::Date => temporal_path(options.temporal, "date", None),
        SqlType::Date32 => temporal_path(options.temporal, "date32", None),
        SqlType::DateTime(_) => temporal_path(options.temporal, "datetime", None),
        SqlType::DateTime64(prec, _) => temporal_path(options.temporal, "datetime64", Some(prec)),
        SqlType::Time => temporal_path(options.temporal, "time", None),
        SqlType::Time64(prec) => temporal_path(options.temporal, "time64", Some(prec)),
        _ => Ok(None),
    }?;

    let base = match base {
        Some(base) => base,
        None => return Ok(None),
    };

    Ok(Some(format!(
        "    #[serde(with = \"{}{}\")]",
        base,
        if is_option { "::option" } else { "" }
    )))
}

fn temporal_path(temporal_mode: Temporal, ty: &str, prec: Option<&u32>) -> Result<Option<String>> {
    let temporal_base = match temporal_mode {
        Temporal::Time => Some("::clickhouse::serde::time::"),
        Temporal::Chrono => Some("::clickhouse::serde::chrono::"),
        Temporal::Raw => None,
    };

    let temporal_base = match temporal_base {
        Some(base) => base,
        None => return Ok(None),
    };

    let prec = match prec {
        None => "",
        Some(&0) => "::secs",
        Some(&3) => "::millis",
        Some(&6) => "::micros",
        Some(&9) => "::nanos",
        Some(p) => {
            bail!(
                "Unsupported precision {} for {} in {:?} mode; supported: 0, 3, 6, 9",
                p,
                ty,
                temporal_mode
            )
        }
    };

    Ok(Some(format!("{}{}{}", temporal_base, ty, prec)))
}

fn make_type(column: &Column, options: &Options) -> Result<String> {
    if is_sentinel(&column.name, options) {
        let (rust_type, _) = sentinel_info(&column.type_)?;
        return Ok(format!("Option<{}>", rust_type));
    }
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
        SqlType::UInt128 => "u128".into(),
        SqlType::Int8 => "i8".into(),
        SqlType::Int16 => "i16".into(),
        SqlType::Int32 => "i32".into(),
        SqlType::Int64 => "i64".into(),
        SqlType::Int128 => "i128".into(),
        SqlType::Bool => "bool".into(),
        SqlType::String if options.owned => "String".into(),
        SqlType::String => "&'a str".into(),
        // SqlType::FixedString(size) => todo!(),
        SqlType::Float32 => "f32".into(),
        SqlType::Float64 => "f64".into(),
        SqlType::Date
        | SqlType::Date32
        | SqlType::DateTime(_)
        | SqlType::DateTime64(_, _)
        | SqlType::Time
        | SqlType::Time64(_) => match options.temporal {
            Temporal::Raw => match sql_type {
                SqlType::Date => "u16".into(),
                SqlType::Date32 => "i32".into(),
                SqlType::DateTime(_) => "u32".into(),
                SqlType::DateTime64(_, _) => "i64".into(),
                SqlType::Time => "i32".into(),
                SqlType::Time64(_) => "i64".into(),
                _ => unreachable!(),
            },
            Temporal::Time => match sql_type {
                SqlType::Date | SqlType::Date32 => "::time::Date".into(),
                SqlType::DateTime(_) => "::time::OffsetDateTime".into(),
                SqlType::DateTime64(..) => "::time::OffsetDateTime".into(),
                SqlType::Time => "::time::Duration".into(),
                SqlType::Time64(_) => "::time::Duration".into(),
                _ => unreachable!(),
            },
            Temporal::Chrono => match sql_type {
                SqlType::Date | SqlType::Date32 => "::chrono::NaiveDate".into(),
                SqlType::DateTime(_) => "::chrono::DateTime<::chrono::Utc>".into(),
                SqlType::DateTime64(..) => "::chrono::DateTime<::chrono::Utc>".into(),
                SqlType::Time => "::chrono::Duration".into(),
                SqlType::Time64(_) => "::chrono::Duration".into(),
                _ => unreachable!(),
            },
        },
        SqlType::IPv4 => "::std::net::Ipv4Addr".into(),
        SqlType::IPv6 => "::std::net::Ipv6Addr".into(),
        SqlType::UUID => "::uuid::Uuid".into(),
        // SqlType::Decimal(_prec, _scale) => todo!(),
        SqlType::Enum8(_) | SqlType::Enum16(_) => name.to_upper_camel_case(),
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
                &column.name.to_upper_camel_case(),
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

    for derive in &options.derives {
        writeln!(dst, "#[derive({})]", derive)?;
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
        name.to_upper_camel_case()
    }
}

fn sentinel_info(sql_type: &SqlType) -> Result<(&'static str, &'static str)> {
    Ok(match sql_type {
        SqlType::String => ("String", r#""""#),
        SqlType::UInt8 => ("u8", "0u8"),
        SqlType::UInt16 => ("u16", "0u16"),
        SqlType::UInt32 => ("u32", "0u32"),
        SqlType::UInt64 => ("u64", "0u64"),
        SqlType::UInt128 => ("u128", "0u128"),
        SqlType::Int8 => ("i8", "0i8"),
        SqlType::Int16 => ("i16", "0i16"),
        SqlType::Int32 => ("i32", "0i32"),
        SqlType::Int64 => ("i64", "0i64"),
        SqlType::Int128 => ("i128", "0i128"),
        SqlType::Float32 => ("f32", "0f32"),
        SqlType::Float64 => ("f64", "0f64"),
        _ => bail!("-N is not supported for type {}", sql_type),
    })
}

fn validate_sentinel(column: &Column, options: &Options) -> Result<()> {
    if matches!(column.type_, SqlType::Nullable(_)) {
        bail!(
            "column `{}` is already Nullable; -N is not needed",
            column.name
        );
    }
    if find_override(&column.name, &column.type_, options).is_some() {
        bail!(
            "column `{}` has both -N and -O/-T; this combination is not supported",
            column.name
        );
    }
    Ok(())
}

fn generate_sentinel_modules(dst: &mut impl Write, table: &Table, options: &Options) -> Result<()> {
    for column in &table.columns {
        if !is_sentinel(&column.name, options) {
            continue;
        }

        validate_sentinel(column, options)?;

        let mod_name = sentinel_mod_name(&column.name);
        let (rust_type, default_lit) = sentinel_info(&column.type_)
            .with_context(|| format!("sentinel column `{}`", column.name))?;

        writeln!(dst, "mod {} {{", mod_name)?;
        writeln!(dst, "    use serde::{{Deserializer, Serializer}};")?;
        writeln!(dst)?;
        writeln!(
            dst,
            "    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<{}>, D::Error>",
            rust_type
        )?;
        writeln!(dst, "    where")?;
        writeln!(dst, "        D: Deserializer<'de>,")?;
        writeln!(dst, "    {{")?;
        writeln!(
            dst,
            "        let value = <{} as serde::Deserialize>::deserialize(deserializer)?;",
            rust_type
        )?;
        writeln!(dst, "        if value == {} {{", default_lit)?;
        writeln!(dst, "            Ok(None)")?;
        writeln!(dst, "        }} else {{")?;
        writeln!(dst, "            Ok(Some(value))")?;
        writeln!(dst, "        }}")?;
        writeln!(dst, "    }}")?;
        writeln!(dst)?;
        writeln!(
            dst,
            "    pub fn serialize<S>(value: &Option<{}>, serializer: S) -> Result<S::Ok, S::Error>",
            rust_type
        )?;
        writeln!(dst, "    where")?;
        writeln!(dst, "        S: Serializer,")?;
        writeln!(dst, "    {{")?;
        writeln!(dst, "        match value {{")?;
        writeln!(
            dst,
            "            Some(v) => serde::Serialize::serialize(v, serializer),",
        )?;
        writeln!(
            dst,
            "            None => serde::Serialize::serialize(&{}, serializer),",
            default_lit
        )?;
        writeln!(dst, "        }}")?;
        writeln!(dst, "    }}")?;
        writeln!(dst, "}}")?;
        writeln!(dst)?;
    }

    Ok(())
}

pub fn generate(table: &Table, options: &Options) -> Result<String> {
    let mut code = String::new();
    generate_prelude(&mut code, options).context("failed to generate a prelude")?;
    writeln!(code)?;
    generate_row(&mut code, table, options).context("failed to generate a row")?;
    writeln!(code)?;
    generate_enums(&mut code, table, options).context("failed to generate enums")?;
    generate_sentinel_modules(&mut code, table, options)
        .context("failed to generate sentinel serde modules")?;
    Ok(code.trim().to_string())
}

#[cfg(test)]
mod tests {
    use structopt::StructOpt;
    use test_case::test_case;

    use super::*;

    fn make_table(columns: Vec<(&str, SqlType)>) -> Table {
        Table {
            columns: columns
                .into_iter()
                .map(|(name, type_)| Column {
                    name: name.into(),
                    type_,
                    comment: String::new(),
                })
                .collect(),
        }
    }

    fn make_options(args: &[&str]) -> Options {
        Options::from_iter(args)
    }

    #[test_case(SqlType::Nullable(Box::new(SqlType::UInt32)) ; "nullable")]
    #[test_case(SqlType::Enum8(vec![])                       ; "enum8")]
    #[test_case(SqlType::Array(Box::new(SqlType::String))    ; "array")]
    #[test_case(SqlType::Date                                ; "date")]
    #[test_case(SqlType::UUID                                ; "uuid")]
    #[test_case(SqlType::Bool                                ; "bool")]
    fn test_sentinel_rejected_for(sql_type: SqlType) {
        let table = make_table(vec![("col", sql_type)]);
        let options = make_options(&["ch2rs", "t", "-S", "-D", "-N", "col"]);
        assert!(generate(&table, &options).is_err());
    }

    #[test_case(SqlType::String,  "Option<String>" ; "string")]
    #[test_case(SqlType::UInt8,   "Option<u8>"     ; "u8")]
    #[test_case(SqlType::UInt16,  "Option<u16>"    ; "u16")]
    #[test_case(SqlType::UInt32,  "Option<u32>"    ; "u32")]
    #[test_case(SqlType::UInt64,  "Option<u64>"    ; "u64")]
    #[test_case(SqlType::UInt128, "Option<u128>"   ; "u128")]
    #[test_case(SqlType::Int8,    "Option<i8>"     ; "i8")]
    #[test_case(SqlType::Int16,   "Option<i16>"    ; "i16")]
    #[test_case(SqlType::Int32,   "Option<i32>"    ; "i32")]
    #[test_case(SqlType::Int64,   "Option<i64>"    ; "i64")]
    #[test_case(SqlType::Int128,  "Option<i128>"   ; "i128")]
    #[test_case(SqlType::Float32, "Option<f32>"    ; "f32")]
    #[test_case(SqlType::Float64, "Option<f64>"    ; "f64")]
    fn test_sentinel_generates(sql_type: SqlType, expected_type: &str) {
        let table = make_table(vec![("col", sql_type)]);
        let options = make_options(&["ch2rs", "t", "-S", "-D", "--owned", "-N", "col"]);
        let code = generate(&table, &options).unwrap();
        assert!(code.contains(&format!("pub col: {}", expected_type)));
        assert!(code.contains("mod sentinel_col"));
    }
}
