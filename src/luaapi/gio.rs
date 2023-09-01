use futures::{AsyncBufReadExt, AsyncReadExt};
use gtk::{
    gdk::AppLaunchContext,
    gio::{
        prelude::*, AppInfo, AppInfoMonitor, File, FileCreateFlags, InputStream,
        InputStreamAsyncBufRead, OutputStream, SocketClient, SocketConnection, Subprocess,
        UnixSocketAddress,
    },
    glib::{Bytes, PRIORITY_DEFAULT},
};
use mlua::prelude::*;
use paste::paste;
use std::{ffi::OsStr, path::Path};

use crate::macros::register_signals;
use crate::utils::catch_lua_errors;

use super::wrappers::SubprocessFlagsWrapper;

fn add_async_read_buf_api(lua: &Lua) -> LuaResult<()> {
    lua.register_userdata_type::<InputStreamAsyncBufRead<InputStream>>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("InputStreamAsyncBufRead<InputStream> {}")
        });

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
    })?;

    Ok(())
}

fn add_streams_api(lua: &Lua) -> LuaResult<()> {
    lua.register_userdata_type::<InputStream>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("InputStream {}")
        });

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
    })?;

    lua.register_userdata_type::<OutputStream>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("OutputStream {}")
        });

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
    })?;

    Ok(())
}

fn add_subprocess_api(lua: &Lua, gio_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<Subprocess>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("Subprocess {}")
        });

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
    })?;
    let subprocess = lua.create_table()?;
    subprocess.set(
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
    gio_table.set("Subprocess", subprocess)?;

    Ok(())
}

pub fn add_socket_api(lua: &Lua, gio_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<SocketConnection>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("SocketConnection {}")
        });

        reg.add_method("input_stream", |lua, this, ()| {
            lua.create_any_userdata(this.input_stream())
        });

        reg.add_method("output_stream", |lua, this, ()| {
            lua.create_any_userdata(this.output_stream())
        });

        reg.add_async_method("close", |_, this, ()| async move {
            this.close_future(PRIORITY_DEFAULT).await.into_lua_err()
        });
    })?;

    lua.register_userdata_type::<SocketClient>(|reg| {
        reg.add_async_method("connect_unix", |lua, this, path: String| async move {
            let address = UnixSocketAddress::new(Path::new(&path));
            let conn = this.connect_future(&address).await.into_lua_err()?;
            lua.create_any_userdata(conn)
        });
    })?;
    let socket_client = lua.create_table()?;
    socket_client.set(
        "new",
        lua.create_function(|lua, ()| {
            let socket = SocketClient::new();
            lua.create_any_userdata(socket)
        })?,
    )?;
    gio_table.set("SocketClient", socket_client)?;

    Ok(())
}

fn add_file_api(lua: &Lua, gio_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<File>(|reg| {
        /* reg.add_async_method(
            "query_info",
            |lua, this, (attributes, flags): (String,Option<LuaUserDataRef<FileQueryInfoFlags>>)| async move {
                let flags = flags.as_deref().unwrap_or(&FileQueryInfoFlags::NONE);
                this.query_info_future(&attributes, *flags, PRIORITY_DEFAULT).await.into_lua_err()?;
                Ok(())
            },
        ); */
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, this, ()| {
            lua.create_string(format!("File {{ path = {:?} }}", this.path()))
        });

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
    })?;
    let file = lua.create_table()?;
    file.set(
        "for_path",
        lua.create_function(|lua, path: String| lua.create_any_userdata(File::for_path(path)))?,
    )?;
    gio_table.set("File", file)?;

    Ok(())
}

fn add_app_info_monitor_api(lua: &Lua, gio_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<AppInfoMonitor>(|reg| {
        register_signals!(reg, [changed]);

        reg.add_meta_method(LuaMetaMethod::ToString, |lua, _, ()| {
            lua.create_string("AppInfoMonitor {}")
        });
    })?;
    let app_info_monitor = lua.create_table()?;
    app_info_monitor.set(
        "get",
        lua.create_function(|lua, ()| lua.create_any_userdata(AppInfoMonitor::get()))?,
    )?;
    gio_table.set("AppInfoMonitor", app_info_monitor)?;

    Ok(())
}

fn add_app_info_api(lua: &Lua, gio_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<AppInfo>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, this, ()| {
            lua.create_string(format!("AppInfo {{ name = \"{}\" }}", this.name()))
        });

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
    })?;
    let app_info = lua.create_table()?;
    app_info.set(
        "all",
        lua.create_function(|lua, ()| {
            AppInfo::all()
                .into_iter()
                .map(|x| lua.create_any_userdata(x))
                .collect::<LuaResult<Vec<_>>>()
        })?,
    )?;
    gio_table.set("AppInfo", app_info)?;

    Ok(())
}

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let gio_table = lua.create_table()?;

    add_async_read_buf_api(lua)?;
    add_streams_api(lua)?;
    add_subprocess_api(lua, &gio_table)?;
    add_socket_api(lua, &gio_table)?;
    add_file_api(lua, &gio_table)?;
    add_app_info_monitor_api(lua, &gio_table)?;
    add_app_info_api(lua, &gio_table)?;

    Ok(gio_table)
}
