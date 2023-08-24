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
