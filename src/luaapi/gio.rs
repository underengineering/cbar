use futures::{AsyncBufReadExt, AsyncReadExt};
use gtk::{
    gio::{
        prelude::*, InputStream, InputStreamAsyncBufRead, OutputStream, SocketClient,
        SocketConnection, Subprocess, SubprocessFlags, UnixSocketAddress,
    },
    glib::{Bytes, PRIORITY_DEFAULT},
};
use mlua::prelude::*;
use paste::paste;
use std::{ffi::OsStr, path::Path};

use crate::utils::pack_mask;

fn add_async_read_buf_api(lua: &Lua) -> LuaResult<()> {
    lua.register_userdata_type::<InputStreamAsyncBufRead<InputStream>>(|reg| {
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
            this.read(&mut buffer).await.into_lua_err()?;
            lua.create_string(&buffer)
        });

        reg.add_async_method_mut("read_to_end", |lua, this, size: usize| async move {
            let mut buffer = vec![0u8; size];
            this.read_to_end(&mut buffer).await.into_lua_err()?;
            lua.create_string(&buffer)
        });
    })?;

    Ok(())
}

fn add_streams_api(lua: &Lua) -> LuaResult<()> {
    lua.register_userdata_type::<InputStream>(|reg| {
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
    let subprocess_flags = lua.create_table()?;
    subprocess_flags.set(
        "new",
        lua.create_function(|lua, flags_table: LuaTable| {
            let mut flags = SubprocessFlags::empty();
            pack_mask!(
                flags_table,
                flags,
                SubprocessFlags,
                [
                    STDIN_PIPE,
                    STDIN_INHERIT,
                    STDOUT_PIPE,
                    STDOUT_SILENCE,
                    STDERR_PIPE,
                    STDERR_SILENCE,
                    STDERR_MERGE,
                    INHERIT_FDS
                ]
            );
            lua.create_any_userdata(flags)
        })?,
    )?;
    gio_table.set("SubprocessFlags", subprocess_flags)?;

    lua.register_userdata_type::<Subprocess>(|reg| {
        reg.add_async_method(
            "communicate_raw",
            |lua, this, data: Option<String>| async move {
                let (stdout, stderr) = this
                    .communicate_future(data.map(|x| Bytes::from(x.as_bytes())).as_ref())
                    .await
                    .into_lua_err()?;
                if stdout.is_some() && stderr.is_some() {
                    let stdout = lua.create_string(stdout.as_deref().unwrap())?;
                    let stderr = lua.create_string(stderr.as_deref().unwrap())?;
                    let values = LuaMultiValue::from_vec(vec![
                        LuaValue::String(stdout),
                        LuaValue::String(stderr),
                    ]);

                    Ok(values)
                } else {
                    Ok(LuaMultiValue::new())
                }
            },
        );

        reg.add_async_method(
            "communicate",
            |lua, this, data: Option<String>| async move {
                let (stdout, stderr) = this.communicate_utf8_future(data).await.into_lua_err()?;
                if stdout.is_some() && stderr.is_some() {
                    let stdout = lua.create_string(stdout.unwrap().as_str())?;
                    let stderr = lua.create_string(stderr.unwrap().as_str())?;
                    let values = LuaMultiValue::from_vec(vec![
                        LuaValue::String(stdout),
                        LuaValue::String(stderr),
                    ]);

                    Ok(values)
                } else {
                    Ok(LuaMultiValue::new())
                }
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
            |lua, (args, flags): (Vec<String>, LuaUserDataRef<SubprocessFlags>)| {
                let proc =
                    Subprocess::newv(&args.iter().map(OsStr::new).collect::<Vec<_>>(), *flags)
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

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let gio_table = lua.create_table()?;

    add_async_read_buf_api(lua)?;
    add_streams_api(lua)?;
    add_subprocess_api(lua, &gio_table)?;
    add_socket_api(lua, &gio_table)?;

    Ok(gio_table)
}
