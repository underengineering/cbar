use async_channel::{Receiver, Sender, TryRecvError, TrySendError};
use mlua::prelude::*;

mod error;

#[allow(clippy::module_inception)]
mod worker;

use crate::traits::LuaApi;

use self::worker::{Worker, WorkerData, WorkerEvent};

impl LuaApi for Sender<WorkerData> {
    const CLASS_NAME: &'static str = "Sender<WorkerData>";
    const CONSTRUCTIBLE: bool = false;

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_async_method("send", |lua, this, value: LuaValue| async {
            this.send(WorkerData::from_lua(value, lua)?)
                .await
                .into_lua_err()
        });

        reg.add_method("send_blocking", |lua, this, value: LuaValue| {
            this.send_blocking(WorkerData::from_lua(value, lua)?)
                .into_lua_err()
        });

        reg.add_method("try_send", |lua, this, value: LuaValue| {
            match this.try_send(WorkerData::from_lua(value, lua)?) {
                Ok(_) => Ok(true),
                Err(TrySendError::Full(_)) => Ok(false),
                Err(err) => Err(err).into_lua_err()?,
            }
        });
    }
}

impl LuaApi for Receiver<WorkerEvent> {
    const CLASS_NAME: &'static str = "Receiver<WorkerEvent>";
    const CONSTRUCTIBLE: bool = false;

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_async_method("recv", |lua, this, ()| async {
            Ok(match this.recv().await.into_lua_err()? {
                WorkerEvent::UserData(data) => data.into_lua(lua)?,
                WorkerEvent::Error(err) => Err(err)?,
                WorkerEvent::Done => LuaValue::Nil, // TODO: Throw a error?
            })
        });

        reg.add_method("recv_blocking", |lua, this, ()| {
            Ok(match this.recv_blocking().into_lua_err()? {
                WorkerEvent::UserData(data) => data.into_lua(lua)?,
                WorkerEvent::Error(err) => Err(err)?,
                WorkerEvent::Done => LuaValue::Nil, // TODO: Throw a error?
            })
        });

        reg.add_method("try_recv", |lua, this, ()| {
            Ok(match this.try_recv() {
                Ok(value) => match value {
                    WorkerEvent::UserData(value) => {
                        LuaMultiValue::from_vec(vec![LuaValue::Boolean(true), value.into_lua(lua)?])
                    }
                    WorkerEvent::Done => Err(TryRecvError::Closed).into_lua_err()?,
                    WorkerEvent::Error(err) => Err(err)?,
                },
                Err(TryRecvError::Empty) => LuaMultiValue::from_vec(vec![LuaValue::Boolean(false)]),
                Err(err) => Err(err).into_lua_err()?,
            })
        });
    }
}

impl LuaApi for Worker {
    const CLASS_NAME: &'static str = "Worker";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("dead", |_, this, ()| Ok(this.dead()));

        reg.add_async_method_mut("join", |lua, this, ()| async {
            if this.dead() {
                // Prevent deadlocking
                Ok(LuaValue::Nil)
            } else {
                let receiver = this.receiver();
                let results = lua.create_table()?;
                loop {
                    match receiver.recv().await.into_lua_err()? {
                        WorkerEvent::UserData(data) => results.push(data)?,
                        WorkerEvent::Error(err) => Err(err).into_lua_err()?,
                        WorkerEvent::Done => break,
                    };
                }

                Ok(LuaValue::Table(results))
            }
        });

        reg.add_method_mut("join_blocking", |lua, this, ()| {
            if this.dead() {
                // Prevent deadlocking
                Ok(LuaValue::Nil)
            } else {
                let receiver = this.receiver();
                let results = lua.create_table()?;
                loop {
                    match receiver.recv_blocking().into_lua_err()? {
                        WorkerEvent::UserData(data) => results.push(data)?,
                        WorkerEvent::Error(err) => Err(err).into_lua_err()?,
                        WorkerEvent::Done => break,
                    };
                }

                Ok(LuaValue::Table(results))
            }
        });

        reg.add_method_mut("sender", |lua, this, ()| {
            lua.create_any_userdata(this.sender())
        });

        reg.add_method_mut("receiver", |lua, this, ()| {
            lua.create_any_userdata(this.receiver())
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "start",
            lua.create_function(
                |lua, (code, name, channel_size): (String, Option<String>, Option<usize>)| {
                    let worker = Worker::start(code, name, channel_size).into_lua_err()?;
                    lua.create_any_userdata(worker)
                },
            )?,
        )?;

        Ok(())
    }
}

pub fn push_api(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
    let worker_table = lua.create_table()?;

    Sender::<WorkerData>::push_lua(lua, &worker_table)?;
    Receiver::<WorkerEvent>::push_lua(lua, &worker_table)?;
    Worker::push_lua(lua, &worker_table)?;

    table.set("worker", worker_table)?;

    Ok(())
}
