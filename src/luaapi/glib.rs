use gtk::glib::{self, Bytes, MainContext, Value};
use mlua::prelude::*;
use paste::paste;

use crate::{traits::LuaApi, utils::catch_lua_errors_async};

fn push_constants(lua: &Lua, glib_table: &LuaTable) -> LuaResult<()> {
    let priority = lua.create_table()?;
    priority.set("HIGH", lua.create_any_userdata(glib::Priority::HIGH)?)?;
    priority.set("DEFAULT", lua.create_any_userdata(glib::Priority::DEFAULT)?)?;
    priority.set(
        "HIGH_IDLE",
        lua.create_any_userdata(glib::Priority::HIGH_IDLE)?,
    )?;
    priority.set(
        "DEFAULT_IDLE",
        lua.create_any_userdata(glib::Priority::DEFAULT_IDLE)?,
    )?;
    priority.set("LOW", lua.create_any_userdata(glib::Priority::LOW)?)?;
    glib_table.set("Priority", priority)?;

    Ok(())
}

impl LuaApi for Value {
    const CLASS_NAME: &'static str = "Value";

    fn to_lua_string<'a>(&self, lua: &'a Lua) -> LuaResult<LuaString<'a>> {
        lua.create_string(format!("Value {:?}", self))
    }

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("type_name", |lua, this, ()| {
            lua.create_string(this.type_().name())
        });

        reg.add_method("as_str", |lua, this, ()| {
            Ok(if let Ok(str) = this.get::<&str>() {
                Some(lua.create_string(str)?)
            } else {
                None
            })
        });

        reg.add_method("as_str_vec", |lua, this, ()| {
            Ok(if let Ok(vec) = this.get::<Vec<String>>() {
                let table = lua.create_table_with_capacity(vec.len(), 0)?;
                for (idx, str) in vec.iter().enumerate() {
                    table.set(idx + 1, lua.create_string(str)?)?;
                }

                Some(table)
            } else {
                None
            })
        });

        reg.add_method("as_bytes", |lua, this, ()| {
            Ok(if let Ok(bytes) = this.get::<Bytes>() {
                Some(lua.create_string(bytes)?)
            } else {
                None
            })
        });

        macro_rules! add_conversions {
            ($reg:ident, [$($typ:ty),+]) => {
                $(
                    $reg.add_method(paste!(stringify!([<as_ $typ>])), |_, this, ()| {
                        Ok(if let Ok(value) = this.get::<$typ>() {
                            Some(value)
                        } else {
                            None
                        })
                    });
                )+
            };
        }

        add_conversions!(reg, [bool, f64, f32, i64, i32, i8, u64, u32, u8]);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new_str",
            lua.create_function(|lua, str: String| lua.create_any_userdata(Value::from(str)))?,
        )?;

        table.set(
            "new_str_vec",
            lua.create_function(|lua, table: LuaTable| {
                let vec = table
                    .sequence_values::<String>()
                    .collect::<LuaResult<Vec<_>>>()?;
                lua.create_any_userdata(Value::from(vec))
            })?,
        )?;

        table.set(
            "new_bytes",
            lua.create_function(|lua, str: LuaString| {
                let bytes = Bytes::from(str.as_bytes());
                lua.create_any_userdata(Value::from(bytes))
            })?,
        )?;

        macro_rules! add_conversions {
            ($table:ident, [$($typ:ty),+]) => {
                $(
                    $table.set(
                        paste!(stringify!([<new_ $typ>])),
                        lua.create_function(|lua, value: $typ| lua.create_any_userdata(Value::from(value)))?,
                    )?;
                )+
            };
        }

        add_conversions!(table, [bool, f64, f32, i64, i32, i8, u64, u32, u8]);

        Ok(())
    }
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
    Value::push_lua(lua, &glib_table)?;
    MainContext::push_lua(lua, &glib_table)?;

    table.set("glib", glib_table)?;

    Ok(())
}
