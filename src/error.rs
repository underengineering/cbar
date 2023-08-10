use mlua::prelude::*;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Lua error")]
    LuaError(#[from] LuaError),
    #[error("I/O error")]
    IoError(#[from] io::Error),
    #[error("Config file does not exist")]
    ConfigFileDoesNotExist,
}
