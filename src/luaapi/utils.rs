use gtk::glib;
use mlua::prelude::*;
use regex::{Captures, Match, Regex};
use std::time::Duration;

use crate::traits::LuaApi;

use super::json::push_json_api;

fn push_grass_api(lua: &Lua, utils_table: &LuaTable) -> LuaResult<()> {
    utils_table.set(
        "scss_from_path",
        lua.create_function(|_, path: String| {
            grass::from_path(path, &grass::Options::default()).into_lua_err()
        })?,
    )?;

    utils_table.set(
        "scss_from_string",
        lua.create_function(|_, input: String| {
            grass::from_string(input, &grass::Options::default()).into_lua_err()
        })?,
    )?;

    Ok(())
}

fn push_icons_api(lua: &Lua, utils_table: &LuaTable) -> LuaResult<()> {
    utils_table.set(
        "lookup_icon",
        lua.create_function(|_, (name, options): (String, Option<LuaTable>)| {
            let mut builder = freedesktop_icons::lookup(&name);
            let mut theme = None;
            if let Some(options) = options {
                if let Some(size) = options.get::<_, Option<u16>>("size")? {
                    builder = builder.with_size(size);
                }

                if let Some(scale) = options.get::<_, Option<u16>>("scale")? {
                    builder = builder.with_scale(scale);
                }

                if let Some(theme_inner) = options.get::<_, Option<String>>("theme")? {
                    // Move it to prevent borrow issues
                    theme = Some(theme_inner);
                }

                if options.get::<_, Option<bool>>("cache")?.unwrap_or(false) {
                    builder = builder.with_cache();
                }

                if options
                    .get::<_, Option<bool>>("force_svg")?
                    .unwrap_or(false)
                {
                    builder = builder.force_svg();
                }
            }

            if let Some(ref theme) = theme {
                builder = builder.with_theme(theme);
            }

            if let Some(path) = builder.find() {
                let path_str = path.into_os_string().into_string().unwrap();
                Ok(Some(path_str))
            } else {
                Ok(None)
            }
        })?,
    )?;

    Ok(())
}

fn push_tokio_api(lua: &Lua, utils_table: &LuaTable) -> LuaResult<()> {
    utils_table.set(
        "sleep",
        lua.create_async_function(|_, secs: f64| async move {
            glib::timeout_future(Duration::from_secs_f64(secs)).await;
            Ok(())
        })?,
    )?;

    Ok(())
}

fn push_other_api(lua: &Lua, utils_table: &LuaTable) -> LuaResult<()> {
    utils_table.set(
        "print_table",
        lua.load(
            r#"
            local tbl, seen, depth = ...
            seen = seen or {}
            depth = depth or 0

            seen[tbl] = true

            local keys = {}
            for k, v in pairs(tbl) do
                keys[#keys + 1] = k
            end

            table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)

            io.write(("\t"):rep(depth))
            io.write("{\n")
            for i = 1, #keys do
                local k = keys[i]
                local v = tbl[k]

                local k_formatted
                local ktype = type(k)
                if ktype == "string" then
                    k_formatted = ("\"%s\""):format(k:gsub("\n", "\\n"))
                else
                    k_formatted = k
                end

                local v_formatted
                local vtype = type(v)
                if vtype == "table" and not seen[v] then
                    io.write(("\t"):rep(depth + 1))
                    io.write(("[%s] =\n"):format(k_formatted))
                    utils.print_table(v, seen, depth + 1)
                    io.write(",\n")
                    goto continue
                elseif vtype == "string" then
                    v_formatted = ("\"%s\""):format(v:gsub("\n", "\\n"))
                else
                    v_formatted = v
                end

                io.write(("\t"):rep(depth + 1))
                io.write(("[%s] = %s,\n"):format(k_formatted, v_formatted))
                ::continue::
            end
            io.write(("\t"):rep(depth))
            io.write("}\n")
            "#,
        )
        .into_function()?,
    )?;

    Ok(())
}

struct MatchWrapper<'h>(Match<'h>);
impl<'lua, 'h> IntoLua<'lua> for MatchWrapper<'h> {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table_with_capacity(0, 4)?;
        table.set("start_index", self.0.start())?;
        table.set("end_index", self.0.end())?;
        table.set("is_empty", self.0.is_empty())?;
        table.set("str", self.0.as_str())?;

        Ok(LuaValue::Table(table))
    }
}

struct CapturesWrapper<'h>(Captures<'h>);
impl<'lua, 'h> CapturesWrapper<'h> {
    fn into_lua(self, re: &Regex, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table_with_capacity(self.0.len(), self.0.len())?;

        let captures_iter = self.0.iter();
        let capture_names_iter = re.capture_names();
        for (idx, (mat, group_name)) in captures_iter.zip(capture_names_iter).enumerate() {
            if let Some(mat) = mat {
                let match_table = MatchWrapper(mat).into_lua(lua)?;
                table.set(idx + 1, match_table.clone())?;
                if let Some(group_name) = group_name {
                    table.set(group_name, match_table)?;
                }
            }
        }

        Ok(LuaValue::Table(table))
    }
}

impl LuaApi for Regex {
    const CLASS_NAME: &'static str = "Regex";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method(
            "is_match",
            |_, this, (haystack, start_index): (String, Option<usize>)| {
                Ok(this.is_match_at(&haystack, start_index.unwrap_or(0)))
            },
        );

        reg.add_method(
            "find",
            |lua, this, (haystack, start_index): (String, Option<usize>)| {
                Ok(
                    if let Some(mat) = this.find_at(&haystack, start_index.unwrap_or(0)) {
                        Some(MatchWrapper(mat).into_lua(lua)?)
                    } else {
                        None
                    },
                )
            },
        );

        reg.add_method(
            "captures",
            |lua, this, (haystack, start_index): (String, Option<usize>)| {
                Ok(
                    if let Some(captures) = this.captures_at(&haystack, start_index.unwrap_or(0)) {
                        Some(CapturesWrapper(captures).into_lua(this, lua)?)
                    } else {
                        None
                    },
                )
            },
        );

        reg.add_method(
            "replace_all",
            |lua, this, (haystack, rep): (String, LuaValue)| {
                if let LuaValue::Function(f) = rep {
                    let mut new = String::with_capacity(haystack.len());
                    let mut last_match = 0;
                    for caps in this.captures_iter(&haystack) {
                        let m = caps.get(0).unwrap();
                        new.push_str(&haystack[last_match..m.start()]);

                        let caps = CapturesWrapper(caps).into_lua(this, lua);
                        let replacement = f.call::<_, String>(caps)?;
                        new.push_str(&replacement);

                        last_match = m.end();
                    }

                    new.push_str(&haystack[last_match..]);
                    lua.create_string(new)
                } else if let LuaValue::String(fmt) = rep {
                    let new = this.replace_all(&haystack, fmt.to_str()?);
                    lua.create_string(&*new)
                } else {
                    Err(LuaError::FromLuaConversionError {
                        from: rep.type_name(),
                        to: "rep",
                        message: None,
                    })
                }
            },
        );
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, re: String| {
                lua.create_any_userdata(Regex::new(&re).into_lua_err()?)
            })?,
        )?;

        Ok(())
    }
}

pub fn push_api(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
    let utils_table = lua.create_table()?;

    push_grass_api(lua, &utils_table)?;
    push_icons_api(lua, &utils_table)?;
    push_tokio_api(lua, &utils_table)?;
    push_json_api(lua, &utils_table)?;
    push_other_api(lua, &utils_table)?;
    Regex::push_lua(lua, &utils_table)?;

    table.set("utils", utils_table)?;

    Ok(())
}
