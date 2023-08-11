use crate::hyprland::{
    event_loop::EventLoop,
    events::Event,
    ipc::{self, commands::*},
};
use mlua::prelude::*;
use tokio::sync::broadcast;

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
            this.connect()
                .await
                .expect("Failed to connect the event loop");
            Ok(())
        });

        reg.add_method_mut("subscribe", |lua, this, ()| {
            let receiver = this.subscribe();
            lua.create_any_userdata(receiver)
        });

        reg.add_async_method_mut("run", |_, this, ()| async move {
            this.run().await.unwrap();
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
