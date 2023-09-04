use mlua::prelude::*;
use paste::paste;
use tokio::sync::broadcast::{self, Receiver};

use crate::{
    hyprland::{
        event_loop::EventLoop,
        events::Event,
        ipc::{self, commands::*},
    },
    traits::LuaApi,
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

fn push_ipc_api(lua: &Lua, hyprland_table: &LuaTable) -> LuaResult<()> {
    let ipc = lua.create_table()?;
    push_ipc_methods!(lua, ipc, [Workspaces, Devices, ActiveWindow, Monitors]);
    hyprland_table.set("ipc", ipc)?;

    Ok(())
}

impl LuaApi for EventLoop {
    const CLASS_NAME: &'static str = "EventLoop";

    fn to_lua_string<'a>(&self, lua: &'a Lua) -> LuaResult<LuaString<'a>> {
        lua.create_string(format!("EventLoop {{ connected = {} }}", self.connected()))
    }

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
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
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let event_loop = EventLoop::new();
                lua.create_any_userdata(event_loop)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for Receiver<Event> {
    const CLASS_NAME: &'static str = "Receiver<Event>";
    const CONSTRUCTIBLE: bool = false;

    fn to_lua_string<'a>(&self, lua: &'a Lua) -> LuaResult<LuaString<'a>> {
        lua.create_string(format!("Receiver<Event> {{ len = {} }}", self.len()))
    }

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("len", |_, this, ()| Ok(this.len()));

        reg.add_async_method_mut("recv", |lua, this, ()| async move {
            // Return nil when channel is lagged
            let ret = match this.recv().await {
                Ok(event) => lua.to_value(&event)?,
                Err(broadcast::error::RecvError::Lagged(_)) => LuaValue::Nil,
                Err(err) => Err(err).into_lua_err()?,
            };

            Ok(ret)
        });
    }
}

pub fn push_api(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
    let hyprland_table = lua.create_table()?;

    push_ipc_api(lua, &hyprland_table)?;
    Receiver::<Event>::push_lua(lua, &hyprland_table)?;
    EventLoop::push_lua(lua, &hyprland_table)?;

    table.set("hyprland", hyprland_table)?;

    Ok(())
}
