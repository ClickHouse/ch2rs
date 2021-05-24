use crate::schema::SqlType;

#[derive(Debug)]
pub struct Options {
    pub url: String,
    pub user: Option<String>,
    pub password: Option<String>,

    pub database: String,
    pub table: String,

    pub owned: bool,
    pub types: Vec<Type>,
    pub overrides: Vec<Override>,
}

#[derive(Debug)]
pub struct Override {
    pub column: String,
    pub type_: String,
}

#[derive(Debug)]
pub struct Type {
    pub sql: SqlType,
    pub type_: String,
}
