use mlua::prelude::*;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Lua error")]
    Lua(#[from] LuaError),
    #[error("I/O error")]
    Io(#[from] io::Error),
    #[error("Config file does not exist")]
    ConfigFileDoesNotExist,
    #[error("Config file is not a file")]
    ConfigFileNotAFile,
}
