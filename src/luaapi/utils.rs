use gtk::glib;
use mlua::prelude::*;
use std::time::Duration;

fn add_grass_api(lua: &Lua, utils_table: &LuaTable) -> LuaResult<()> {
    utils_table.set(
        "scss_from_path",
        lua.create_function(|_, path: String| {
            Ok(grass::from_path(path, &grass::Options::default())
                .expect("Failed to load scss file"))
        })?,
    )?;

    utils_table.set(
        "scss_from_string",
        lua.create_function(|_, input: String| {
            Ok(grass::from_string(input, &grass::Options::default())
                .expect("Failed to load scss string"))
        })?,
    )?;

    Ok(())
}

fn add_icons_api(lua: &Lua, utils_table: &LuaTable) -> LuaResult<()> {
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

fn add_tokio_api(lua: &Lua, utils_table: &LuaTable) -> LuaResult<()> {
    utils_table.set(
        "sleep",
        lua.create_async_function(|_, secs: f64| async move {
            glib::timeout_future(Duration::from_secs_f64(secs)).await;
            Ok(())
        })?,
    )?;

    Ok(())
}

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let utils_table = lua.create_table()?;

    add_grass_api(lua, &utils_table)?;
    add_icons_api(lua, &utils_table)?;
    add_tokio_api(lua, &utils_table)?;

    Ok(utils_table)
}
