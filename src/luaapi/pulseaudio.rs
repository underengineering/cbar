use gtk::glib::MainContext;
use mlua::{prelude::*, IntoLua};

use crate::utils::catch_lua_errors;

use super::wrappers::InterestMaskSetWrapper;

macro_rules! push_enum {
    ($tbl:ident, $name:ty, [$($variant:ident),+]) => {
        $($tbl.set(stringify!($variant), <$name>::$variant as i32)?;)+
    };
}

macro_rules! copy_field {
    ($tbl:ident, $data:expr, $name:ident) => {
        $tbl.set(stringify!($name), $data.$name)?;
    };
}

macro_rules! copy_field_wrapped {
    ($tbl:ident, $data:expr, $wrapper:expr, $name:ident) => {
        $tbl.set(stringify!($name), $wrapper($data.$name))?;
    };
}

struct VolumeWrapper(pulse::volume::Volume);
impl<'lua> IntoLua<'lua> for VolumeWrapper {
    fn into_lua(self, _: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(LuaValue::Integer(self.0 .0 as i64))
    }
}

struct ChannelVolumesWrapper(pulse::volume::ChannelVolumes);
impl<'lua> IntoLua<'lua> for ChannelVolumesWrapper {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table_with_capacity(self.0.len() as usize, 0)?;
        for (idx, volume) in self.0.get().iter().enumerate() {
            table.set(idx + 1, volume.0 as i64)?;
        }

        Ok(LuaValue::Table(table))
    }
}

struct SinkInfoWrapper<'a>(&'a pulse::context::introspect::SinkInfo<'a>);
impl<'lua, 'a> IntoLua<'lua> for SinkInfoWrapper<'a> {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table_with_capacity(0, 5)?;
        table.set("name", self.0.name.to_owned())?;
        copy_field!(table, self.0, index);
        copy_field_wrapped!(table, self.0, ChannelVolumesWrapper, volume);
        copy_field!(table, self.0, mute);
        copy_field_wrapped!(table, self.0, VolumeWrapper, base_volume);

        Ok(LuaValue::Table(table))
    }
}

struct ServerInfoWrapper<'a>(&'a pulse::context::introspect::ServerInfo<'a>);
impl<'lua, 'a> IntoLua<'lua> for ServerInfoWrapper<'a> {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table()?;
        table.set("user_name", self.0.user_name.to_owned())?;
        table.set("host_name", self.0.host_name.to_owned())?;
        table.set("server_version", self.0.server_version.to_owned())?;
        table.set("server_name", self.0.server_name.to_owned())?;
        table.set("default_sink_name", self.0.default_sink_name.to_owned())?;
        table.set("default_source_name", self.0.default_source_name.to_owned())?;

        Ok(LuaValue::Table(table))
    }
}

fn add_enums(lua: &Lua, pulseaudio_table: &LuaTable) -> LuaResult<()> {
    let state = lua.create_table()?;
    push_enum!(
        state,
        pulse::context::State,
        [
            Unconnected,
            Connecting,
            Authorizing,
            SettingName,
            Ready,
            Failed,
            Terminated
        ]
    );
    pulseaudio_table.set("State", state)?;

    let facility = lua.create_table()?;
    push_enum!(
        facility,
        pulse::context::subscribe::Facility,
        [
            Sink,
            Source,
            SinkInput,
            SourceOutput,
            Module,
            Client,
            SampleCache,
            Server,
            Card
        ]
    );
    pulseaudio_table.set("Facility", facility)?;

    let operation = lua.create_table()?;
    push_enum!(
        operation,
        pulse::context::subscribe::Operation,
        [New, Changed, Removed]
    );
    pulseaudio_table.set("Operation", operation)?;

    Ok(())
}

fn add_mainloop_api(lua: &Lua, pulseaudio_table: &LuaTable) -> LuaResult<()> {
    let mainloop = lua.create_table()?;
    mainloop.set(
        "new",
        lua.create_function(|lua, ctx: Option<LuaUserDataRefMut<MainContext>>| {
            let mainloop = if let Some(mut udref) = ctx {
                pulse_glib::Mainloop::new(Some(&mut *udref))
            } else {
                pulse_glib::Mainloop::new(None)
            };

            if let Some(mainloop) = mainloop {
                Ok(Some(lua.create_any_userdata(mainloop)?))
            } else {
                Ok(None)
            }
        })?,
    )?;
    pulseaudio_table.set("Mainloop", mainloop)?;

    Ok(())
}

fn add_context_api(lua: &Lua, pulseaudio_table: &LuaTable) -> LuaResult<()> {
    let volume = lua.create_table()?;
    volume.set("NORMAL", pulse::volume::Volume::NORMAL.0)?;
    volume.set("MUTED", pulse::volume::Volume::MUTED.0)?;
    volume.set("MAX", pulse::volume::Volume::MAX.0)?;
    volume.set("INVALID", pulse::volume::Volume::INVALID.0)?;
    pulseaudio_table.set("Volume", volume)?;

    lua.register_userdata_type::<pulse::context::Context>(|reg| {
        reg.add_meta_method(LuaMetaMethod::ToString, |lua, this, ()| {
            lua.create_string(format!("Context {{ state = {:?} }}", this.get_state()))
        });

        reg.add_method_mut("connect", |_, this, server: Option<String>| {
            let server = server.as_deref();
            this.connect(server, pulse::context::FlagSet::NOAUTOSPAWN, None)
                .into_lua_err()?;

            Ok(())
        });

        reg.add_method_mut("set_state_callback", |_, this, f: LuaOwnedFunction| {
            this.set_state_callback(Some(Box::new(move || {
                catch_lua_errors::<_, ()>(f.to_ref(), ());
            })));

            Ok(())
        });

        reg.add_method("get_state", |_, this, ()| {
            let state = this.get_state();
            Ok(state as i32)
        });

        reg.add_method_mut(
            "subscribe",
            |_, this, (mask, f): (InterestMaskSetWrapper, LuaOwnedFunction)| {
                this.subscribe(mask.0, move |success| {
                    catch_lua_errors::<_, ()>(f.to_ref(), success);
                });

                Ok(())
            },
        );

        reg.add_method_mut("set_subscribe_callback", |_, this, f: LuaOwnedFunction| {
            this.set_subscribe_callback(Some(Box::new(move |facility, operation, index| {
                let facility = facility.map(|value| value as i32);
                let operation = operation.map(|value| value as i32);
                catch_lua_errors::<_, ()>(f.to_ref(), (facility, operation, index));
            })));

            Ok(())
        });

        reg.add_method("get_server_info", |_, this, f: LuaOwnedFunction| {
            this.introspect().get_server_info(move |result| {
                catch_lua_errors::<_, ()>(f.to_ref(), ServerInfoWrapper(result));
            });

            Ok(())
        });

        reg.add_method(
            "get_sink_info_by_index",
            |_, this, (index, f): (i64, LuaOwnedFunction)| {
                this.introspect()
                    .get_sink_info_by_index(index as u32, move |result| {
                        if let pulse::callbacks::ListResult::Item(item) = result {
                            catch_lua_errors::<_, ()>(f.to_ref(), SinkInfoWrapper(item));
                        }
                    });

                Ok(())
            },
        );

        reg.add_method(
            "get_sink_info_by_name",
            |_, this, (name, f): (String, LuaOwnedFunction)| {
                this.introspect()
                    .get_sink_info_by_name(&name, move |result| {
                        if let pulse::callbacks::ListResult::Item(item) = result {
                            catch_lua_errors::<_, ()>(f.to_ref(), SinkInfoWrapper(item));
                        }
                    });

                Ok(())
            },
        );
    })?;
    let context = lua.create_table()?;
    context.set(
        "new",
        lua.create_function(
            |lua, (mainloop, name): (LuaUserDataRef<pulse_glib::Mainloop>, String)| {
                let ctx = pulse::context::Context::new(&*mainloop, &name);
                if let Some(ctx) = ctx {
                    Ok(Some(lua.create_any_userdata(ctx)?))
                } else {
                    Ok(None)
                }
            },
        )?,
    )?;
    pulseaudio_table.set("Context", context)?;

    Ok(())
}

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let pulseaudio_table = lua.create_table()?;

    add_enums(lua, &pulseaudio_table)?;
    add_mainloop_api(lua, &pulseaudio_table)?;
    add_context_api(lua, &pulseaudio_table)?;

    Ok(pulseaudio_table)
}
