use futures::{AsyncBufReadExt, AsyncReadExt};
use gtk::{
    gdk::AppLaunchContext,
    gio::{
        prelude::*, AppInfo, AppInfoMonitor, File, FileCreateFlags, Icon, InputStream,
        InputStreamAsyncBufRead, OutputStream, SocketClient, SocketConnection, Subprocess,
        ThemedIcon, UnixSocketAddress,
    },
    glib::{Bytes, GString, PRIORITY_DEFAULT},
};
use mlua::prelude::*;
use paste::paste;
use std::{ffi::OsStr, path::Path};

use crate::utils::catch_lua_errors;
use crate::{macros::register_signals, traits::LuaApi};

use super::wrappers::SubprocessFlagsWrapper;

impl LuaApi for InputStreamAsyncBufRead<InputStream> {
    const CLASS_NAME: &'static str = "InputStreamAsyncBufRead<InputStream>";
    const CONSTRUCTIBLE: bool = false;

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_async_method_mut("read_line", |_, this, capacity: Option<usize>| async move {
            let mut buffer = capacity.map_or_else(String::new, String::with_capacity);
            this.read_line(&mut buffer).await.into_lua_err()?;
            Ok(buffer)
        });

        reg.add_async_method_mut("read_exact", |lua, this, size: usize| async move {
            let mut buffer = vec![0u8; size];
            this.read_exact(&mut buffer).await.into_lua_err()?;
            lua.create_string(&buffer)
        });

        reg.add_async_method_mut("read", |lua, this, size: usize| async move {
            let mut buffer = vec![0u8; size];
            let read = this.read(&mut buffer).await.into_lua_err()?;
            lua.create_string(&buffer[..read])
        });

        reg.add_async_method_mut("read_to_end", |lua, this, size: usize| async move {
            let mut buffer = vec![0u8; size];
            let read = this.read_to_end(&mut buffer).await.into_lua_err()?;
            lua.create_string(&buffer[..read])
        });
    }
}

impl LuaApi for InputStream {
    const CLASS_NAME: &'static str = "InputStream";
    const CONSTRUCTIBLE: bool = false;

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_function(
            "into_async_buf_read",
            |lua, (this, buffer_size): (LuaOwnedAnyUserData, usize)| {
                let this = this.take::<InputStream>()?;
                lua.create_any_userdata(this.into_async_buf_read(buffer_size))
            },
        );

        reg.add_async_method("read", |lua, this, count: usize| async move {
            let data = this
                .read_bytes_future(count, PRIORITY_DEFAULT)
                .await
                .into_lua_err()?;
            lua.create_string(data)
        });

        reg.add_async_method("skip", |_, this, count: usize| async move {
            let read = this
                .skip_future(count, PRIORITY_DEFAULT)
                .await
                .into_lua_err()?;
            Ok(read)
        });

        reg.add_async_method("close", |_, this, ()| async move {
            this.close_future(PRIORITY_DEFAULT).await.into_lua_err()?;
            Ok(())
        });
    }
}

impl LuaApi for OutputStream {
    const CLASS_NAME: &'static str = "OutputStream";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_async_method("write", |_, this, data: LuaString| async move {
            let buffer = Bytes::from(data.as_bytes());
            let written = this
                .write_bytes_future(&buffer, PRIORITY_DEFAULT)
                .await
                .into_lua_err()?;
            Ok(written)
        });

        reg.add_async_method("flush", |_, this, ()| async move {
            this.flush_future(PRIORITY_DEFAULT).await.into_lua_err()?;
            Ok(())
        });

        reg.add_method("has_pending", |_, this, ()| Ok(this.has_pending()));
        reg.add_method("is_closed", |_, this, ()| Ok(this.is_closed()));
        reg.add_method("is_closing", |_, this, ()| Ok(this.is_closing()));

        reg.add_async_method("close", |_, this, ()| async move {
            this.close_future(PRIORITY_DEFAULT).await.into_lua_err()?;
            Ok(())
        });
    }
}

impl LuaApi for Subprocess {
    const CLASS_NAME: &'static str = "Subprocess";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_async_method(
            "communicate_raw",
            |lua, this, data: Option<String>| async move {
                let (stdout, stderr) = this
                    .communicate_future(data.map(|x| Bytes::from(x.as_bytes())).as_ref())
                    .await
                    .into_lua_err()?;

                let values = vec![
                    match stdout {
                        Some(stdout) => LuaValue::String(lua.create_string(&*stdout)?),
                        None => LuaValue::Nil,
                    },
                    match stderr {
                        Some(stderr) => LuaValue::String(lua.create_string(&*stderr)?),
                        None => LuaValue::Nil,
                    },
                ];

                Ok(LuaMultiValue::from_vec(values))
            },
        );

        reg.add_async_method(
            "communicate",
            |lua, this, data: Option<String>| async move {
                let (stdout, stderr) = this.communicate_utf8_future(data).await.into_lua_err()?;

                let values = vec![
                    match stdout {
                        Some(stdout) => LuaValue::String(lua.create_string(&*stdout)?),
                        None => LuaValue::Nil,
                    },
                    match stderr {
                        Some(stderr) => LuaValue::String(lua.create_string(&*stderr)?),
                        None => LuaValue::Nil,
                    },
                ];

                Ok(LuaMultiValue::from_vec(values))
            },
        );

        reg.add_async_method("wait", |_, this, ()| async move {
            this.wait_future().await.into_lua_err()?;

            Ok(())
        });

        reg.add_method("send_signal", |_, this, signal_num: i32| {
            this.send_signal(signal_num);
            Ok(())
        });
        reg.add_method("identifier", |_, this, ()| {
            Ok(this.identifier().map(String::from))
        });
        reg.add_method("force_kill", |_, this, ()| {
            this.force_exit();
            Ok(())
        });
        reg.add_method("has_exited", |_, this, ()| Ok(this.has_exited()));
        reg.add_method("exit_status", |_, this, ()| Ok(this.exit_status()));

        reg.add_method("stdin", |lua, this, ()| {
            if let Some(stdin) = this.stdin_pipe() {
                Ok(Some(lua.create_any_userdata(stdin)?))
            } else {
                Ok(None)
            }
        });
        reg.add_method("stdout", |lua, this, ()| {
            if let Some(stdout) = this.stdout_pipe() {
                Ok(Some(lua.create_any_userdata(stdout)?))
            } else {
                Ok(None)
            }
        });
        reg.add_method("stderr", |lua, this, ()| {
            if let Some(stderr) = this.stderr_pipe() {
                Ok(Some(lua.create_any_userdata(stderr)?))
            } else {
                Ok(None)
            }
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(
                |lua, (args, flags): (Vec<String>, SubprocessFlagsWrapper)| {
                    let proc =
                        Subprocess::newv(&args.iter().map(OsStr::new).collect::<Vec<_>>(), flags.0)
                            .into_lua_err()?;

                    lua.create_any_userdata(proc)
                },
            )?,
        )?;

        Ok(())
    }
}

impl LuaApi for SocketConnection {
    const CLASS_NAME: &'static str = "SocketConnection";
    const CONSTRUCTIBLE: bool = false;

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("input_stream", |lua, this, ()| {
            lua.create_any_userdata(this.input_stream())
        });

        reg.add_method("output_stream", |lua, this, ()| {
            lua.create_any_userdata(this.output_stream())
        });

        reg.add_async_method("close", |_, this, ()| async move {
            this.close_future(PRIORITY_DEFAULT).await.into_lua_err()
        });
    }
}

impl LuaApi for SocketClient {
    const CLASS_NAME: &'static str = "SocketClient";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_async_method("connect_unix", |lua, this, path: String| async move {
            let address = UnixSocketAddress::new(Path::new(&path));
            let conn = this.connect_future(&address).await.into_lua_err()?;
            lua.create_any_userdata(conn)
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let socket = SocketClient::new();
                lua.create_any_userdata(socket)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for File {
    const CLASS_NAME: &'static str = "File";

    fn to_lua_string<'a>(&self, lua: &'a Lua) -> LuaResult<LuaString<'a>> {
        lua.create_string(format!("File {{ path = {:?} }}", self.path()))
    }

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("path", |lua, this, ()| {
            let result = if let Some(path) = this.path() {
                Some(lua.create_string(path.into_os_string().to_str().unwrap())?)
            } else {
                None
            };

            Ok(result)
        });

        reg.add_async_method("read", |lua, this, ()| async move {
            let stream = this.read_future(PRIORITY_DEFAULT).await.into_lua_err()?;
            lua.create_any_userdata(stream.upcast::<InputStream>())
        });

        reg.add_async_method("create", |lua, this, ()| async move {
            let stream = this
                .create_future(FileCreateFlags::NONE, PRIORITY_DEFAULT)
                .await
                .into_lua_err()?;
            lua.create_any_userdata(stream.upcast::<OutputStream>())
        });

        reg.add_async_method("replace", |lua, this, ()| async move {
            let stream = this
                .replace_future(None, false, FileCreateFlags::NONE, PRIORITY_DEFAULT)
                .await
                .into_lua_err()?;
            lua.create_any_userdata(stream.upcast::<OutputStream>())
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "for_path",
            lua.create_function(|lua, path: String| lua.create_any_userdata(File::for_path(path)))?,
        )?;

        Ok(())
    }
}

impl LuaApi for Icon {
    const CLASS_NAME: &'static str = "Icon";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("as_themed", |lua, this, ()| {
            Ok(
                if let Ok(themed_icon) = this.clone().downcast::<ThemedIcon>() {
                    Some(lua.create_any_userdata(themed_icon)?)
                } else {
                    None
                },
            )
        });
    }
}

impl LuaApi for ThemedIcon {
    const CLASS_NAME: &'static str = "ThemedIcon";

    fn to_lua_string<'a>(&self, lua: &'a Lua) -> LuaResult<LuaString<'a>> {
        lua.create_string(format!("ThemedIcon {{ names = {:?} }}", self.names()))
    }

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("names", |lua, this, ()| {
            let names = this.names();
            lua.create_sequence_from(names.iter().map(GString::as_str))
        })
    }
}

impl LuaApi for AppInfoMonitor {
    const CLASS_NAME: &'static str = "AppInfoMonitor";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        register_signals!(reg, [changed]);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "get",
            lua.create_function(|lua, ()| lua.create_any_userdata(AppInfoMonitor::get()))?,
        )?;

        Ok(())
    }
}

impl LuaApi for AppInfo {
    const CLASS_NAME: &'static str = "AppInfo";

    fn to_lua_string<'a>(&self, lua: &'a Lua) -> LuaResult<LuaString<'a>> {
        lua.create_string(format!("AppInfo {{ name = \"{}\" }}", self.name()))
    }

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("name", |lua, this, ()| {
            lua.create_string(this.name().as_str())
        });

        reg.add_method("display_name", |lua, this, ()| {
            lua.create_string(this.display_name().as_str())
        });

        reg.add_method("icon", |lua, this, ()| {
            let result = if let Some(icon) = this.icon() {
                Some(lua.create_any_userdata(icon)?)
            } else {
                None
            };

            Ok(result)
        });

        reg.add_method("id", |lua, this, ()| {
            let result = if let Some(id) = this.id() {
                Some(lua.create_string(id.as_str())?)
            } else {
                None
            };

            Ok(result)
        });

        reg.add_method("description", |lua, this, ()| {
            let result = if let Some(description) = this.description() {
                Some(lua.create_string(description.as_str())?)
            } else {
                None
            };

            Ok(result)
        });

        reg.add_method("delete", |_, this, ()| Ok(this.delete()));
        reg.add_method("launch", |_, this, files: Vec<LuaUserDataRef<File>>| {
            this.launch(
                &files.into_iter().map(|x| x.clone()).collect::<Vec<_>>(),
                None::<&AppLaunchContext>,
            )
            .into_lua_err()?;
            Ok(())
        });
        reg.add_method("launch_uris", |_, this, uris: Vec<String>| {
            this.launch_uris(
                &uris.iter().map(String::as_str).collect::<Vec<_>>(),
                None::<&AppLaunchContext>,
            )
            .into_lua_err()?;
            Ok(())
        });

        reg.add_method("should_show", |_, this, ()| Ok(this.should_show()));
        reg.add_method("supports_files", |_, this, ()| Ok(this.supports_files()));
        reg.add_method("supports_uris", |_, this, ()| Ok(this.supports_uris()));
        reg.add_method("can_delete", |_, this, ()| Ok(this.can_delete()));
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "all",
            lua.create_function(|lua, ()| {
                AppInfo::all()
                    .into_iter()
                    .map(|x| lua.create_any_userdata(x))
                    .collect::<LuaResult<Vec<_>>>()
            })?,
        )?;

        Ok(())
    }
}

pub fn push_api(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
    let gio_table = lua.create_table()?;
    InputStreamAsyncBufRead::<InputStream>::push_lua(lua, &gio_table)?;
    InputStream::push_lua(lua, &gio_table)?;
    OutputStream::push_lua(lua, &gio_table)?;
    Subprocess::push_lua(lua, &gio_table)?;
    SocketConnection::push_lua(lua, &gio_table)?;
    SocketClient::push_lua(lua, &gio_table)?;
    File::push_lua(lua, &gio_table)?;
    Icon::push_lua(lua, &gio_table)?;
    ThemedIcon::push_lua(lua, &gio_table)?;
    AppInfoMonitor::push_lua(lua, &gio_table)?;
    AppInfo::push_lua(lua, &gio_table)?;
    table.set("gio", gio_table)?;

    Ok(())
}
