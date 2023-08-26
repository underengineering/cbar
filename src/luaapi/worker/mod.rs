use crossbeam::channel::{Receiver, Sender};
use mlua::prelude::*;

mod error;

#[allow(clippy::module_inception)]
mod worker;

use self::worker::{Worker, WorkerData, WorkerEvent};

fn add_worker_api(lua: &Lua, worker_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<Sender<WorkerData>>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("Sender<WorkerData {}")
        });

        reg.add_method("send", |lua, this, value: LuaValue| {
            this.send(WorkerData::from_lua(value, lua)?).into_lua_err()
        })
    })?;

    lua.register_userdata_type::<Receiver<WorkerEvent>>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("Receiver<WorkerData> {}")
        });

        reg.add_method("recv", |lua, this, ()| {
            Ok(match this.recv().into_lua_err()? {
                WorkerEvent::UserData(data) => data.into_lua(lua)?,
                WorkerEvent::Error(err) => Err(err)?,
                WorkerEvent::Done => LuaValue::Nil,
            })
        })
    })?;

    lua.register_userdata_type::<Worker>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("Worker {}")
        });

        reg.add_method("dead", |_, this, ()| Ok(this.dead()));

        reg.add_method("join", |_, this, ()| {
            let receiver = this.receiver();
            loop {
                match receiver.recv().into_lua_err()? {
                    WorkerEvent::Done => break,
                    WorkerEvent::Error(err) => Err(err).into_lua_err()?,
                    _ => {}
                }
            }

            Ok(())
        });

        reg.add_method("sender", |lua, this, ()| {
            lua.create_any_userdata(this.sender())
        });

        reg.add_method("receiver", |lua, this, ()| {
            lua.create_any_userdata(this.receiver())
        });
    })?;

    let worker = lua.create_table()?;
    worker.set(
        "start",
        lua.create_function(|lua, (code, name): (String, Option<String>)| {
            let worker = Worker::start(code, name).into_lua_err()?;
            lua.create_any_userdata(worker)
        })?,
    )?;
    worker_table.set("Worker", worker)?;
    Ok(())
}

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let worker_table = lua.create_table()?;

    add_worker_api(lua, &worker_table)?;

    Ok(worker_table)
}
