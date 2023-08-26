use std::{
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
};

use mlua::prelude::*;

use super::error::Error;
use crate::luaapi::{gio, gtk, hyprland, pulseaudio, sysinfo, utf8, utils};

pub enum WorkerData {
    Nil,
    Boolean(bool),
    Number(f64),
    Integer(i64),
    String(String),
    Array(Vec<WorkerData>),
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
            LuaValue::UserData(_) => Err(LuaError::FromLuaConversionError {
                from: "userdata",
                to: "WorkerData",
                message: None,
            }),
        }
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
    receiver: Rc<Receiver<WorkerEvent>>,
}

impl Worker {
    pub fn start(code: String, name: Option<String>) -> Result<Self, Error> {
        // worker -> main
        let (tx_, rx) = mpsc::channel();
        // main -> worker
        let (tx, rx_) = mpsc::channel();

        let dead = Arc::new(AtomicBool::new(false));
        let dead_ref = dead.clone();
        std::thread::spawn(move || {
            let lua = unsafe { Lua::unsafe_new() };
            lua.load_from_std_lib(LuaStdLib::ALL).unwrap();

            Self::setup_env(&lua, tx_.clone(), rx_).unwrap();

            match lua
                .load(code)
                .set_name(name.as_deref().unwrap_or("worker"))
                .exec()
            {
                Ok(_) => tx_.send(WorkerEvent::Done),
                Err(err) => tx_.send(WorkerEvent::Error(err)),
            }
            .unwrap();

            dead_ref.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        Ok(Self {
            dead,
            sender: tx,
            receiver: Rc::new(rx),
        })
    }

    pub fn dead(&self) -> bool {
        self.dead.load(Ordering::Relaxed)
    }

    pub fn sender(&self) -> Sender<WorkerData> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> &Rc<Receiver<WorkerEvent>> {
        &self.receiver
    }

    fn add_channels_api(lua: &Lua) -> LuaResult<()> {
        lua.register_userdata_type::<Sender<WorkerEvent>>(|reg| {
            reg.add_method("send", |lua, this, data: LuaValue| {
                this.send(WorkerEvent::UserData(WorkerData::from_lua(data, lua)?))
                    .into_lua_err()
            });
        })?;

        lua.register_userdata_type::<Receiver<WorkerData>>(|reg| {
            reg.add_method("recv", |_, this, ()| this.recv().into_lua_err());
        })?;

        Ok(())
    }

    fn setup_env(
        lua: &Lua,
        sender: Sender<WorkerEvent>,
        receiver: Receiver<WorkerData>,
    ) -> LuaResult<()> {
        let globals = lua.globals();
        globals.set("gio", gio::add_api(lua)?)?;
        globals.set("gtk", gtk::add_api(lua)?)?;
        globals.set("hyprland", hyprland::add_api(lua)?)?;
        globals.set("pulseaudio", pulseaudio::add_api(lua)?)?;
        globals.set("sysinfo", sysinfo::add_api(lua)?)?;
        globals.set("utf8", utf8::add_api(lua)?)?;
        globals.set("utils", utils::add_api(lua)?)?;

        Self::add_channels_api(lua)?;

        let worker_table = lua.create_table()?;
        worker_table.set("sender", lua.create_any_userdata(sender)?)?;
        worker_table.set("receiver", lua.create_any_userdata(receiver)?)?;
        globals.set("worker", worker_table)?;

        Ok(())
    }
}
