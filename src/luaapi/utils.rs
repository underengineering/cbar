use mlua::prelude::*;

fn add_grass_api(lua: &Lua, utils_table: &LuaTable) -> LuaResult<()> {
    utils_table.set(
        "scss_from_path",
        lua.create_function(|_, path: String| {
            grass::from_path(path, &grass::Options::default()).unwrap();
            Ok(())
        })?,
    )?;

    utils_table.set(
        "scss_from_string",
        lua.create_function(|_, input: String| {
            grass::from_string(input, &grass::Options::default()).unwrap();
            Ok(())
        })?,
    )?;

    Ok(())
}

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let utils_table = lua.create_table()?;

    add_grass_api(lua, &utils_table)?;

    Ok(utils_table)
}
