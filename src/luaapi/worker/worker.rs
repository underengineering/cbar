use ::gtk::gdk::Texture;
use async_channel::{Receiver, Sender, TryRecvError, TrySendError};
use mlua::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use super::error::Error;
use crate::{
    luaapi::{gdk, gio, glib, gtk, hyprland, pulseaudio, sysinfo, utf8, utils},
    traits::{LuaApi, LuaExt},
};

pub enum WorkerData {
    Nil,
    Boolean(bool),
    Number(f64),
    Integer(i64),
    String(String),
    Array(Vec<WorkerData>),

    Texture(Texture),
}

impl<'lua> IntoLua<'lua> for WorkerData {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(match self {
            Self::Nil => LuaValue::Nil,
            Self::Boolean(value) => LuaValue::Boolean(value),
            Self::Number(value) => LuaValue::Number(value),
            Self::Integer(value) => LuaValue::Integer(value),
            Self::String(str) => LuaValue::String(lua.create_string(str)?),
            Self::Array(arr) => {
                let result = lua.create_table_with_capacity(arr.len(), 0)?;
                for (idx, value) in arr.into_iter().enumerate() {
                    result.set(idx + 1, value)?;
                }

                LuaValue::Table(result)
            }
            Self::Texture(texture) => LuaValue::UserData(lua.create_any_userdata(texture)?),
        })
    }
}

impl<'lua> FromLua<'lua> for WorkerData {
    fn from_lua(value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Nil => Ok(Self::Nil),
            LuaValue::Boolean(value) => Ok(Self::Boolean(value)),
            LuaValue::LightUserData(_) => Err(LuaError::FromLuaConversionError {
                from: "lightuserdata",
                to: "WorkerData",
                message: None,
            }),
            LuaValue::Integer(value) => Ok(Self::Integer(value)),
            LuaValue::Number(value) => Ok(Self::Number(value)),
            // TODO:
            LuaValue::Error(_) => Err(LuaError::FromLuaConversionError {
                from: "error",
                to: "WorkerData",
                message: None,
            }),
            LuaValue::String(str) => Ok(Self::String(String::from(str.to_str()?))),
            LuaValue::Table(table) => {
                let mut vec = Vec::with_capacity(table.raw_len());
                for value in table.sequence_values::<LuaValue>() {
                    vec.push(Self::from_lua(value?, _lua)?);
                }

                Ok(Self::Array(vec))
            }
            LuaValue::Function(_) => Err(LuaError::FromLuaConversionError {
                from: "function",
                to: "WorkerData",
                message: None,
            }),
            LuaValue::Thread(_) => Err(LuaError::FromLuaConversionError {
                from: "thread",
                to: "WorkerData",
                message: None,
            }),
            LuaValue::UserData(ud) => {
                if let Ok(texture) = ud.take::<Texture>() {
                    Ok(Self::Texture(texture))
                } else {
                    Err(LuaError::FromLuaConversionError {
                        from: "userdata",
                        to: "WorkerData",
                        message: None,
                    })
                }
            }
        }
    }
}

impl LuaApi for Sender<WorkerEvent> {
    const CLASS_NAME: &'static str = "Sender<WorkerEvent>";
    const CONSTRUCTIBLE: bool = false;

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_async_method("send", |lua, this, data: LuaValue| async {
            let data = WorkerEvent::UserData(WorkerData::from_lua(data, lua)?);
            this.send(data).await.into_lua_err()
        });

        reg.add_method("send_blocking", |lua, this, data: LuaValue| {
            let data = WorkerEvent::UserData(WorkerData::from_lua(data, lua)?);
            this.send_blocking(data).into_lua_err()
        });

        reg.add_method("try_send", |lua, this, data: LuaValue| {
            let data = WorkerEvent::UserData(WorkerData::from_lua(data, lua)?);
            match this.try_send(data) {
                Ok(_) => Ok(true),
                Err(TrySendError::Full(_)) => Ok(false),
                Err(err) => Err(err).into_lua_err()?,
            }
        });
    }
}

impl LuaApi for Receiver<WorkerData> {
    const CLASS_NAME: &'static str = "Receiver<WorkerData>";
    const CONSTRUCTIBLE: bool = false;

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_async_method("recv", |_, this, ()| async {
            this.recv().await.into_lua_err()
        });

        reg.add_method("recv_blocking", |_, this, ()| {
            this.recv_blocking().into_lua_err()
        });

        reg.add_method("try_recv", |lua, this, ()| {
            Ok(match this.try_recv() {
                Ok(value) => {
                    LuaMultiValue::from_vec(vec![LuaValue::Boolean(false), value.into_lua(lua)?])
                }
                Err(TryRecvError::Empty) => LuaMultiValue::from_vec(vec![LuaValue::Boolean(false)]),
                Err(err) => Err(err).into_lua_err()?,
            })
        });
    }
}

pub enum WorkerEvent {
    UserData(WorkerData),
    Error(LuaError),
    Done,
}

pub struct Worker {
    dead: Arc<AtomicBool>,
    sender: Sender<WorkerData>,
    receiver: Receiver<WorkerEvent>,
}

impl Worker {
    pub fn start(
        code: String,
        name: Option<String>,
        channel_size: Option<usize>,
    ) -> Result<Self, Error> {
        // worker -> main
        let (tx_, rx) = async_channel::bounded(channel_size.unwrap_or(32));
        // main -> worker
        let (tx, rx_) = async_channel::bounded(channel_size.unwrap_or(32));

        let dead = Arc::new(AtomicBool::new(false));
        let dead_ref = dead.clone();
        std::thread::spawn(move || {
            let lua = unsafe { Lua::new_with_stock_allocator() };
            lua.load_from_std_lib(LuaStdLib::ALL).unwrap();

            Self::setup_env(&lua, tx_.clone(), rx_).unwrap();

            match lua
                .load(code)
                .set_name(name.as_deref().unwrap_or("worker"))
                .exec()
            {
                Ok(_) => tx_.send_blocking(WorkerEvent::Done),
                Err(err) => tx_.send_blocking(WorkerEvent::Error(err)),
            }
            .unwrap();

            dead_ref.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        Ok(Self {
            dead,
            sender: tx,
            receiver: rx,
        })
    }

    pub fn dead(&self) -> bool {
        self.dead.load(Ordering::Relaxed)
    }

    pub fn sender(&mut self) -> Sender<WorkerData> {
        self.sender.clone()
    }

    pub fn receiver(&mut self) -> Receiver<WorkerEvent> {
        self.receiver.clone()
    }

    fn add_channels_api(lua: &Lua, worker_table: &LuaTable) -> LuaResult<()> {
        Sender::<WorkerEvent>::push_lua(lua, worker_table)?;
        Receiver::<WorkerData>::push_lua(lua, worker_table)?;

        Ok(())
    }

    fn setup_env(
        lua: &Lua,
        sender: Sender<WorkerEvent>,
        receiver: Receiver<WorkerData>,
    ) -> LuaResult<()> {
        let globals = lua.globals();
        let crabshell_table = lua.create_table()?;
        gdk::push_api(lua, &crabshell_table)?;
        gio::push_api(lua, &crabshell_table)?;
        glib::push_api(lua, &crabshell_table)?;
        gtk::push_api(lua, &crabshell_table)?;
        hyprland::push_api(lua, &crabshell_table)?;
        pulseaudio::push_api(lua, &crabshell_table)?;
        sysinfo::push_api(lua, &crabshell_table)?;
        utf8::push_api(lua, &crabshell_table)?;
        utils::push_api(lua, &crabshell_table)?;

        let worker_table = lua.create_table()?;
        Self::add_channels_api(lua, &worker_table)?;
        worker_table.set("sender", lua.create_any_userdata(sender)?)?;
        worker_table.set("receiver", lua.create_any_userdata(receiver)?)?;
        crabshell_table.set("worker", worker_table)?;
        globals.set("crabshell", crabshell_table)?;

        Ok(())
    }
}
