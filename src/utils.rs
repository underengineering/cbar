use mlua::{prelude::*, Lua};

use crate::error::LuaErrorWrapper;

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

pub fn catch_lua_errors<'lua, A, R>(f: LuaFunction<'lua>, args: A) -> Option<R>
where
    A: IntoLuaMulti<'lua>,
    R: FromLuaMulti<'lua>,
{
    match f.call::<A, R>(args) {
        Ok(value) => Some(value),
        Err(err) => {
            eprintln!("Uncaught lua callback error:\n{}", LuaErrorWrapper(err));
            None
        }
    }
}

pub async fn catch_lua_errors_async<'lua, A, R>(f: LuaFunction<'lua>, args: A) -> Option<R>
where
    A: IntoLuaMulti<'lua>,
    R: FromLuaMulti<'lua> + 'lua,
{
    match f.call_async::<A, R>(args).await {
        Ok(value) => Some(value),
        Err(err) => {
            eprintln!("Uncaught lua callback error:\n{}", LuaErrorWrapper(err));
            None
        }
    }
}
