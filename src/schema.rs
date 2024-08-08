use std::fmt;

#[derive(Debug)]
pub struct Table {
    pub columns: Vec<Column>,
}

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub type_: SqlType,
    pub comment: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum SqlType {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Bool,
    String,
    FixedString(u32),
    Float32,
    Float64,
    Date,
    DateTime(Option<String>),
    DateTime64(u32, Option<String>),
    IPv4,
    IPv6,
    UUID,
    Decimal(u32, u32),
    Enum8(Vec<(String, i32)>),
    Enum16(Vec<(String, i32)>),
    Array(Box<SqlType>),
    Tuple(Vec<SqlType>),
    Map(Box<SqlType>, Box<SqlType>),
    Nullable(Box<SqlType>),
}

impl fmt::Display for SqlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlType::DateTime(Some(tz)) => write!(f, "DateTime({})", tz),
            SqlType::DateTime(None) => f.write_str("DateTime"),
            SqlType::DateTime64(prec, Some(tz)) => write!(f, "DateTime64({}, {})", prec, tz),
            SqlType::DateTime64(prec, None) => write!(f, "DateTime64({})", prec),
            _ => fmt::Debug::fmt(self, f),
        }
    }
}
