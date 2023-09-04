use mlua::{prelude::*, Lua};

pub trait LuaExt {
    unsafe fn new_with_stock_allocator() -> Lua {
        let state = mlua_sys::luaL_newstate();

        mlua_sys::luaL_requiref(
            state,
            "_G" as *const str as *const i8,
            mlua_sys::luaopen_base,
            1,
        );
        mlua_sys::lua_pop(state, 1);

        Lua::init_from_ptr(state)
    }
}

impl LuaExt for Lua {}

pub trait LuaApi
where
    Self: Sized + 'static,
{
    const CLASS_NAME: &'static str;
    const CONSTRUCTIBLE: bool = true;

    fn push_lua(lua: &Lua, namespace: &LuaTable) -> LuaResult<()> {
        lua.register_userdata_type::<Self>(|reg| {
            reg.add_meta_method(LuaMetaMethod::ToString, |lua, this, ()| {
                this.to_lua_string(lua)
            });

            Self::register_methods(reg);
            Self::register_subclasses(reg);
        })?;

        if Self::CONSTRUCTIBLE {
            let table = lua.create_table()?;
            Self::register_static_methods(lua, &table)?;
            namespace.set(Self::CLASS_NAME, table)?;
        }

        Ok(())
    }

    fn to_lua_string<'a>(&self, lua: &'a Lua) -> LuaResult<LuaString<'a>> {
        lua.create_string(format!("{} {{ }}", Self::CLASS_NAME))
    }

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>);

    #[allow(unused_variables)]
    fn register_subclasses(reg: &mut LuaUserDataRegistry<Self>) {}

    #[allow(unused_variables)]
    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        Ok(())
    }
}
