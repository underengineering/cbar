use mlua::prelude::*;
use thiserror::Error;

/// JSON api-specific errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("Lua error")]
    Lua(#[from] LuaError),
    #[error("Cannot convert `{0}` to a serde_json::Value")]
    JsonConversion(String),
    #[error("Cannot convert `{0}` to a JSON number")]
    JsonInvalidNumber(f64),
}
