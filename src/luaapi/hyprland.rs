use crate::hyprland::ipc::{self, commands::*};
use mlua::prelude::*;

macro_rules! ipc_to_lua {
   (match $obj:expr, lua = $lua:expr, buffer = $buffer:expr, [$($command:ty),+]) => {
       match $obj {
           $(<$command>::NAME => $lua.to_value(&ipc::request::<$command>($buffer).await.unwrap())?),*,
            _ => panic!("Unknown ipc '{}'", $obj),
       }
   }
}

fn add_ipc_api(lua: &Lua, hyprland_table: &LuaTable) -> LuaResult<()> {
    hyprland_table.set(
        "ipc_request",
        lua.create_async_function(|lua, name: String| async move {
            let mut buffer = Vec::new();
            let resp = ipc_to_lua! {
                match name.as_str(),
                lua = lua,
                buffer = &mut buffer,
                [Workspaces, Devices, ActiveWindow]
            };

            Ok(resp)
        })?,
    )?;

    Ok(())
}

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let hyprland_table = lua.create_table()?;

    add_ipc_api(lua, &hyprland_table)?;

    Ok(hyprland_table)
}
