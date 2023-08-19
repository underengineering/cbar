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
