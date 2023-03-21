use crate::store::Expiry;

#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, String, Option<Expiry>),
    Echo(String),
    Ping,
}
