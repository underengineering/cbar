use clap::Parser;
use mlua::prelude::*;
use std::{env, fs, path::PathBuf};

mod error;
mod hyprland;
mod luaapi;
mod macros;
mod system_info;
mod traits;
mod utils;

use crate::{
    error::{Error, LuaErrorWrapper},
    traits::LuaExt,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the lua file to execute.
    /// A default config path will be prepended if it's relative
    /// Defaults to $HOME/.config/crabshell/main.lua
    #[arg(short, long)]
    config: Option<PathBuf>,
    /// Values to pass to lua script
    args: Vec<String>,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let config_path = args
        .config
        .map(|path| {
            if path.is_relative() {
                let mut abs_path =
                    PathBuf::from(env::var("HOME").expect("Failed to get the HOME variable"));
                abs_path.push(".config/crabshell");
                abs_path.push(path);

                abs_path
            } else {
                path
            }
        })
        .unwrap_or_else(|| {
            let mut path =
                PathBuf::from(env::var("HOME").expect("Failed to get the HOME variable"));
            path.push(".config/crabshell/main.lua");
            path
        });

    if !config_path.try_exists()? {
        return Err(Error::ConfigFileDoesNotExist);
    }

    if !config_path.is_file() {
        return Err(Error::ConfigFileNotAFile);
    }

    let lua = unsafe { Lua::new_with_stock_allocator() };
    lua.load_from_std_lib(LuaStdLib::ALL)?;

    let globals = lua.globals();
    let crabshell_table = lua.create_table()?;
    luaapi::gio::push_api(&lua, &crabshell_table)?;
    luaapi::glib::push_api(&lua, &crabshell_table)?;
    luaapi::gtk::push_api(&lua, &crabshell_table)?;
    luaapi::hyprland::push_api(&lua, &crabshell_table)?;
    luaapi::pulseaudio::push_api(&lua, &crabshell_table)?;
    luaapi::sysinfo::push_api(&lua, &crabshell_table)?;
    luaapi::utf8::push_api(&lua, &crabshell_table)?;
    luaapi::utils::push_api(&lua, &crabshell_table)?;
    luaapi::worker::push_api(&lua, &crabshell_table)?;
    globals.set("crabshell", crabshell_table)?;

    // Set current directory to the config path
    env::set_current_dir(
        config_path
            .parent()
            .expect("Failed to get config parent directory"),
    )?;

    let config = fs::read_to_string(&config_path)?;
    let file_name = config_path.file_name().unwrap().to_str().unwrap();

    let lua_args = args
        .args
        .into_iter()
        .map(|x| lua.create_string(x).expect("Failed to create string"))
        .map(LuaValue::String);
    let result = lua
        .load(config)
        .set_name(file_name)
        .call::<_, ()>(LuaMultiValue::from_iter(lua_args));
    if let Err(lua_err) = result {
        eprintln!("{}", LuaErrorWrapper(lua_err));
        std::process::exit(1)
    }

    Ok(())
}
