use mlua::prelude::*;

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let utf8_table = lua.create_table()?;

    utf8_table.set(
        "len",
        lua.create_function(|_, str: String| Ok(str.chars().count()))?,
    )?;

    utf8_table.set(
        "sub",
        lua.create_function(|lua, (str, begin, end): (String, usize, Option<usize>)| {
            let mut indices = str.char_indices();
            let end = end.unwrap_or(usize::MAX);
            if begin > end {
                lua.create_string("")
            } else {
                let begin = indices.nth(begin - 1).map_or(str.len(), |x| x.0);
                let end = indices.nth(end - begin).map_or(str.len(), |x| x.0);

                if cfg!(debug_assertions) {
                    lua.create_string(&str[begin..end])
                } else {
                    unsafe { lua.create_string(str.get_unchecked(begin..end)) }
                }
            }
        })?,
    )?;

    Ok(utf8_table)
}
