use mlua::{prelude::*, Lua};

macro_rules! pack_mask {
    ($tbl:ident, $mask:ident, $masktyp:ty, [$($value:ident),+]) => {
        $(
            if $tbl
                .get::<_, Option<bool>>(paste!(stringify!([<$value:lower>])))?
                .unwrap_or(false)
            {
                $mask |= <$masktyp>::$value;
            }
        )+
    };
}

pub(crate) use pack_mask;

macro_rules! unpack_mask_postfixed {
    ($tbl:ident, $mask:expr, $masktyp:ty, [$($value:ident),+], $postfix:ident) => {
        $(
            paste! {
                $tbl.set(stringify!([<$value:lower>]), $mask.contains(<$masktyp>::[<$value $postfix>]))?;
            }
        )+
    };
}

pub(crate) use unpack_mask_postfixed;

macro_rules! register_signals {
    ($reg: ident, [$($signal:ident),+]) => {
    $(
        paste! {
            $reg.add_method(stringify!([<connect_ $signal>]), |_, this, f: LuaOwnedFunction| {
                this.[<connect_ $signal>](move |_| {
                    catch_lua_errors::<_, ()>(f.to_ref(), ());
                });
                Ok(())
            });
        }
    )+
    };
}

pub(crate) use register_signals;

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
