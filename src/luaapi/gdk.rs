use gtk::{gdk::Texture, glib::Bytes};
use mlua::prelude::*;

use crate::traits::LuaApi;

impl LuaApi for Texture {
    const CLASS_NAME: &'static str = "Texture";

    fn register_methods(_reg: &mut LuaUserDataRegistry<Self>) {}

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "from_bytes",
            lua.create_function(|lua, data: LuaString| {
                let texture = Texture::from_bytes(&Bytes::from(data.as_bytes())).into_lua_err()?;
                lua.create_any_userdata(texture)
            })?,
        )?;
        table.set(
            "from_filename",
            lua.create_function(|lua, path: String| {
                let texture = Texture::from_filename(path).into_lua_err()?;
                lua.create_any_userdata(texture)
            })?,
        )?;

        Ok(())
    }
}

pub fn push_api(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
    let gdk_table = lua.create_table()?;

    Texture::push_lua(lua, &gdk_table)?;
    table.set("gdk", gdk_table)?;

    Ok(())
}
