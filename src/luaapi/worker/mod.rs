use crossbeam::channel::{Receiver, Sender, TryRecvError, TrySendError};
use mlua::prelude::*;

mod error;

#[allow(clippy::module_inception)]
mod worker;

use self::worker::{Worker, WorkerData, WorkerEvent};

fn add_worker_api(lua: &Lua, worker_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<Sender<WorkerData>>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("Sender<WorkerData> {}")
        });

        reg.add_method("send", |lua, this, value: LuaValue| {
            this.send(WorkerData::from_lua(value, lua)?).into_lua_err()
        });

        reg.add_method("try_send", |lua, this, value: LuaValue| {
            match this.try_send(WorkerData::from_lua(value, lua)?) {
                Ok(_) => Ok(true),
                Err(TrySendError::Full(_)) => Ok(false),
                Err(err) => Err(err).into_lua_err()?,
            }
        });
    })?;

    lua.register_userdata_type::<Receiver<WorkerEvent>>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("Receiver<WorkerData> {}")
        });

        reg.add_method("recv", |lua, this, ()| {
            Ok(match this.recv().into_lua_err()? {
                WorkerEvent::UserData(data) => data.into_lua(lua)?,
                WorkerEvent::Done => LuaValue::Nil, // TODO: Throw a error?
                WorkerEvent::Error(err) => Err(err)?,
            })
        });

        reg.add_method("try_recv", |lua, this, ()| {
            Ok(match this.try_recv() {
                Ok(value) => match value {
                    WorkerEvent::UserData(value) => {
                        LuaMultiValue::from_vec(vec![LuaValue::Boolean(true), value.into_lua(lua)?])
                    }
                    WorkerEvent::Done => Err(TryRecvError::Disconnected).into_lua_err()?,
                    WorkerEvent::Error(err) => Err(err)?,
                },
                Err(TryRecvError::Empty) => LuaMultiValue::from_vec(vec![LuaValue::Boolean(false)]),
                Err(err) => Err(err).into_lua_err()?,
            })
        });
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
