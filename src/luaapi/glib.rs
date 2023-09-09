use gtk::glib::{self, MainContext};
use mlua::prelude::*;

use crate::{traits::LuaApi, utils::catch_lua_errors_async};

fn push_constants(lua: &Lua, glib_table: &LuaTable) -> LuaResult<()> {
    let priority = lua.create_table()?;
    priority.set("HIGH", lua.create_any_userdata(glib::PRIORITY_HIGH)?)?;
    priority.set("DEFAULT", lua.create_any_userdata(glib::PRIORITY_DEFAULT)?)?;
    priority.set(
        "HIGH_IDLE",
        lua.create_any_userdata(glib::PRIORITY_HIGH_IDLE)?,
    )?;
    priority.set(
        "DEFAULT_IDLE",
        lua.create_any_userdata(glib::PRIORITY_DEFAULT_IDLE)?,
    )?;
    priority.set("LOW", lua.create_any_userdata(glib::PRIORITY_LOW)?)?;
    glib_table.set("Priority", priority)?;

    Ok(())
}

impl LuaApi for MainContext {
    const CLASS_NAME: &'static str = "MainContext";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("spawn_local", |_, this, f: LuaOwnedFunction| {
            this.spawn_local(async move {
                catch_lua_errors_async::<_, ()>(f.to_ref(), ()).await;
            });
            Ok(())
        });

        reg.add_method(
            "spawn_local_with_priority",
            |_, this, (priority, f): (LuaUserDataRef<glib::Priority>, LuaOwnedFunction)| {
                this.spawn_local_with_priority(*priority, async move {
                    catch_lua_errors_async::<_, ()>(f.to_ref(), ()).await;
                });
                Ok(())
            },
        );
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let ctx = MainContext::new();
                lua.create_any_userdata(ctx)
            })?,
        )?;
        table.set(
            "default",
            lua.create_function(|lua, ()| {
                let ctx = MainContext::default();
                lua.create_any_userdata(ctx)
            })?,
        )?;
        table.set(
            "thread_default",
            lua.create_function(|lua, ()| {
                Ok(if let Some(ctx) = MainContext::thread_default() {
                    Some(lua.create_any_userdata(ctx)?)
                } else {
                    None
                })
            })?,
        )?;

        Ok(())
    }
}

pub fn push_api(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
    let glib_table = lua.create_table()?;

    push_constants(lua, &glib_table)?;
    MainContext::push_lua(lua, &glib_table)?;

    table.set("glib", glib_table)?;

    Ok(())
}
