use mlua::prelude::LuaError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Lua error")]
    Lua(#[from] LuaError),
}
