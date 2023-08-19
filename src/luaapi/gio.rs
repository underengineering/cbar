use futures::pin_mut;
use gtk::gio::{prelude::*, InputStream, OutputStream, Subprocess, SubprocessFlags};
use gtk::glib::{Bytes, PRIORITY_DEFAULT};
use mlua::prelude::*;
use paste::paste;
use std::ffi::OsStr;

use crate::utils::pack_mask;

fn add_streams_api(lua: &Lua) -> LuaResult<()> {
    lua.register_userdata_type::<InputStream>(|reg| {
        reg.add_async_method("read", |lua, this, count: usize| async move {
            let data = this
                .read_bytes_future(count, PRIORITY_DEFAULT)
                .await
                .expect("Failed to read from input stream");
            lua.create_string(data)
        });

        reg.add_async_method("skip", |_, this, count: usize| async move {
            let read = this
                .skip_future(count, PRIORITY_DEFAULT)
                .await
                .expect("Failed to read from input stream");
            Ok(read)
        });

        reg.add_async_method("close", |_, this, ()| async move {
            this.close_future(PRIORITY_DEFAULT)
                .await
                .expect("Failed to close input stream");
            Ok(())
        });
    })?;

    lua.register_userdata_type::<OutputStream>(|reg| {
        reg.add_async_method("write", |_, this, data: LuaString| async move {
            let buffer = Bytes::from(data.as_bytes());
            let written = this
                .write_bytes_future(&buffer, PRIORITY_DEFAULT)
                .await
                .expect("Failed to write to output stream");
            Ok(written)
        });

        reg.add_async_method("flush", |_, this, ()| async move {
            this.flush_future(PRIORITY_DEFAULT)
                .await
                .expect("Failed to flush output stream");
            Ok(())
        });

        reg.add_method("has_pending", |_, this, ()| Ok(this.has_pending()));
        reg.add_method("is_closed", |_, this, ()| Ok(this.is_closed()));
        reg.add_method("is_closing", |_, this, ()| Ok(this.is_closing()));

        reg.add_async_method("close", |_, this, ()| async move {
            this.close_future(PRIORITY_DEFAULT)
                .await
                .expect("Failed to close output stream");
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
                    .expect("Failed to communicate with process");
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
                let (stdout, stderr) = this
                    .communicate_utf8_future(data)
                    .await
                    .expect("Failed to communicate with process");
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
            this.wait_future()
                .await
                .expect("Failed to wait for process termination");

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
                        .expect("Failed to create process");

                lua.create_any_userdata(proc)
            },
        )?,
    )?;
    gio_table.set("Subprocess", subprocess)?;

    Ok(())
}

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let gio_table = lua.create_table()?;

    add_streams_api(lua)?;
    add_subprocess_api(lua, &gio_table)?;

    Ok(gio_table)
}
