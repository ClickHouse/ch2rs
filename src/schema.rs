use clickhouse::Reflection;
use serde::Deserialize;

#[derive(Debug)]
pub struct Schema {
    pub tables: Vec<Table>,
}

#[derive(Debug)]
pub struct Table {
    pub database: String,
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub type_: SqlType,
    pub comment: String,
    pub low_cardinality: bool,
}

#[derive(Debug)]
pub enum SqlType {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,
    String,
    FixedString(usize),
    Float32,
    Float64,
    Date,
    DateTime(Option<String>),
    DateTime64(u32, Option<String>),
    Ipv4,
    Ipv6,
    Uuid,
    Array(Box<SqlType>),
    Decimal(u8, u8),
    Enum8(Vec<(String, i8)>),
    Enum16(Vec<(String, i16)>),
    Nullable(Box<SqlType>),
}
