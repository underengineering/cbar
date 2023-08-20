use mlua::prelude::*;
use paste::paste;
use tokio::sync::broadcast;

use crate::hyprland::{
    event_loop::EventLoop,
    events::Event,
    ipc::{self, commands::*},
};

macro_rules! push_ipc_methods {
    ($lua:ident, $table:ident, [$($command:ty),+]) => {
        $(
            let fn_name = paste!(stringify!([<get_ $command:lower>]));
            $table.set(fn_name,
                $lua.create_async_function(|lua, ()| async move {
                    let mut buffer = Vec::new();
                    let resp = ipc::request::<$command>(&mut buffer).await.unwrap();
                    lua.to_value(&resp)
                })?)?;
        )+
    }
}

fn add_ipc_api(lua: &Lua, hyprland_table: &LuaTable) -> LuaResult<()> {
    let ipc = lua.create_table()?;
    push_ipc_methods!(lua, ipc, [Workspaces, Devices, ActiveWindow, Monitors]);
    hyprland_table.set("ipc", ipc)?;

    Ok(())
}

fn add_event_api(lua: &Lua, hyprland_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<broadcast::Receiver<Event>>(|reg| {
        reg.add_async_method_mut("recv", |lua, this, ()| async move {
            // Return nil when channel is lagged
            let ret = match this.recv().await {
                Ok(event) => lua.to_value(&event)?,
                Err(broadcast::error::RecvError::Lagged(_)) => LuaValue::Nil,
                Err(err) => panic!("{:?}", err),
            };

            Ok(ret)
        });
    })?;

    lua.register_userdata_type::<EventLoop>(|reg| {
        reg.add_async_method_mut("connect", |_, this, ()| async move {
            this.connect().await.into_lua_err()?;
            Ok(())
        });

        reg.add_method_mut("subscribe", |lua, this, ()| {
            let receiver = this.subscribe();
            lua.create_any_userdata(receiver)
        });

        reg.add_async_method_mut("run", |_, this, ()| async move {
            this.run().await.into_lua_err()?;
            Ok(())
        });
    })?;
    let event_loop = lua.create_table()?;
    event_loop.set(
        "new",
        lua.create_function(|lua, ()| {
            let event_loop = EventLoop::new();
            lua.create_any_userdata(event_loop)
        })?,
    )?;
    hyprland_table.set("EventLoop", event_loop)?;

    Ok(())
}

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let hyprland_table = lua.create_table()?;

    add_ipc_api(lua, &hyprland_table)?;
    add_event_api(lua, &hyprland_table)?;

    Ok(hyprland_table)
}
