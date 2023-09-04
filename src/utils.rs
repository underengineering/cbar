use mlua::prelude::*;

use crate::error::LuaErrorWrapper;

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
