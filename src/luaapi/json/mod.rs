use mlua::prelude::*;
use serde_json::Value;

mod conversions;
mod error;

use self::conversions::{TryFromJson, TryJsonFrom};

pub fn add_json_api(lua: &Lua, utils_table: &LuaTable) -> LuaResult<()> {
    let json = lua.create_table()?;
    json.set(
        "to_string",
        lua.create_function(|_, table: LuaTable| {
            let value = Value::try_json_from(table).into_lua_err()?;
            serde_json::to_string(&value).into_lua_err()
        })?,
    )?;
    json.set(
        "to_string_pretty",
        lua.create_function(|_, table: LuaTable| {
            let value = Value::try_json_from(table).into_lua_err()?;
            serde_json::to_string_pretty(&value).into_lua_err()
        })?,
    )?;

    json.set(
        "from_str",
        lua.create_function(|lua, str: LuaString| {
            let json_value = serde_json::from_slice(str.as_bytes()).into_lua_err()?;
            LuaValue::try_from_json(lua, &json_value).into_lua_err()
        })?,
    )?;
    utils_table.set("json", json)?;

    Ok(())
}
