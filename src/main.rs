use clap::Parser;
use mlua::prelude::*;
use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    io::{Read, Write},
    path::PathBuf,
};

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
    /// Disable bytecode caching
    #[arg(short = 'u', long, default_value_t = false)]
    disable_bytecode_cache: bool,
    /// Writes stripped bytecode
    #[arg(short, long, default_value_t = false)]
    strip_bytecode: bool,
    /// Delete the cached bytecode file and exit
    #[arg(short, long, default_value_t = false)]
    purge_cached_bytecode: bool,
    /// Values to pass to lua script
    args: Vec<String>,
}

const CACHE_PATH: &str = "/tmp/crabshell";
fn get_cache_path(config_path: &PathBuf) -> PathBuf {
    let mut cache_path = PathBuf::from(CACHE_PATH);

    let mut hasher = DefaultHasher::new();
    config_path.hash(&mut hasher);
    let config_path_hash = hasher.finish();
    let config_path_hash = format!("{:016x}.bc", config_path_hash);
    cache_path.push(config_path_hash);

    cache_path
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

    if args.purge_cached_bytecode {
        let cache_path = get_cache_path(&config_path);
        if cache_path.is_file() {
            fs::remove_file(cache_path)?;
            return Ok(());
        } else {
            std::process::exit(1);
        }
    }

    let lua = unsafe { Lua::new_with_stock_allocator() };
    lua.load_from_std_lib(LuaStdLib::ALL)?;

    let globals = lua.globals();
    let crabshell_table = lua.create_table()?;
    luaapi::gdk::push_api(&lua, &crabshell_table)?;
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

    // Add config path to the package.path
    lua.load(format!(
        r#"package.path = package.path .. ';{}/?.lua'"#,
        config_path.parent().unwrap().to_str().unwrap()
    ))
    .exec()?;

    let config = fs::read_to_string(&config_path)?;
    let file_name = config_path.file_name().unwrap().to_str().unwrap();

    let func = if args.disable_bytecode_cache {
        lua.load(config).set_name(file_name).into_function()?
    } else {
        let cache_path = get_cache_path(&config_path);

        let mut hasher = DefaultHasher::new();
        config.hash(&mut hasher);
        let new_config_hash = hasher.finish();

        let load_config = || -> Result<LuaFunction, Error> {
            // Compile config
            let func = lua.load(config).set_name(file_name).into_function()?;

            // Get bytecode
            let bytecode = func.dump(args.strip_bytecode);

            // Write bytecode to the cache
            fs::create_dir_all(CACHE_PATH)?;

            let cache_path = get_cache_path(&config_path);
            let mut file = fs::File::create(cache_path)?;
            file.write_all(&new_config_hash.to_le_bytes())?;
            file.write_all(&bytecode)?;

            Ok(func)
        };

        // Load bytecode
        if cache_path.is_file() {
            let mut cache_file = fs::File::open(cache_path)?;

            let config_hash = {
                let mut config_hash_data = [0u8; 8];
                cache_file.read_exact(&mut config_hash_data)?;
                u64::from_le_bytes(config_hash_data)
            };

            // Compare hash
            if new_config_hash != config_hash {
                // Hash mismatch: load config from file
                load_config()?
            } else {
                // Load cached bytecode
                let mut bytecode = Vec::new();
                cache_file.read_to_end(&mut bytecode)?;
                lua.load(bytecode).set_name(file_name).into_function()?
            }
        } else {
            // Cache file does not exist: load config from file
            load_config()?
        }
    };

    let lua_args = args
        .args
        .into_iter()
        .map(|x| lua.create_string(x).expect("Failed to create string"))
        .map(LuaValue::String);
    let result = func.call::<_, ()>(LuaMultiValue::from_iter(lua_args));

    if let Err(lua_err) = result {
        eprintln!("{}", LuaErrorWrapper(lua_err));
        std::process::exit(1)
    }

    Ok(())
}
