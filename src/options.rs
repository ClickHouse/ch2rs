#[derive(Debug)]
pub struct Options {
    pub url: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}
