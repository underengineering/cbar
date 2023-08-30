use mlua::Lua;

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
                    f.call::<_, ()>(()).unwrap();
                });
                Ok(())
            });
        }
    )+
    };
}

pub(crate) use register_signals;

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
