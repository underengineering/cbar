use gtk::glib::MainContext;
use mlua::{prelude::*, IntoLua};
use pulse::{
    callbacks::ListResult,
    context::Context,
    volume::{ChannelVolumes, Volume},
};
use pulse_glib::Mainloop;

use crate::{traits::LuaApi, utils::catch_lua_errors};

use super::wrappers::InterestMaskSetWrapper;

macro_rules! push_enum {
    ($lua:ident, $tbl:ident, $lua_name:expr, $name:ty, [$($variant:ident),+]) => {
        let enum_table = $lua.create_table()?;
        $(enum_table.set(stringify!($variant), <$name>::$variant as i32)?;)+
        $tbl.set($lua_name, enum_table)?;
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

struct VolumeWrapper(Volume);
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
            table.set(idx + 1, volume.0)?;
        }

        Ok(LuaValue::Table(table))
    }
}

impl<'lua> FromLua<'lua> for ChannelVolumesWrapper {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            let mut volume = ChannelVolumes::default();
            volume.set_len(table.raw_len() as u8);

            let volume_slice = volume.get_mut();
            for (key, value) in table
                .sequence_values::<u32>()
                .take(u8::MAX as usize)
                .enumerate()
            {
                let value = value?;
                volume_slice[key] = Volume(value);
            }

            Ok(Self(volume))
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: stringify!($typ),
                message: None,
            })
        }
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

fn push_enums(lua: &Lua, pulseaudio_table: &LuaTable) -> LuaResult<()> {
    push_enum!(
        lua,
        pulseaudio_table,
        "State",
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

    push_enum!(
        lua,
        pulseaudio_table,
        "Facility",
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

    push_enum!(
        lua,
        pulseaudio_table,
        "Operation",
        pulse::context::subscribe::Operation,
        [New, Changed, Removed]
    );

    Ok(())
}

impl LuaApi for Mainloop {
    const CLASS_NAME: &'static str = "Mainloop";

    fn register_methods(_reg: &mut LuaUserDataRegistry<Self>) {}

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, mut ctx: Option<LuaUserDataRefMut<MainContext>>| {
                let mainloop = Mainloop::new(ctx.as_deref_mut());
                if let Some(mainloop) = mainloop {
                    Ok(Some(lua.create_any_userdata(mainloop)?))
                } else {
                    Ok(None)
                }
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for Context {
    const CLASS_NAME: &'static str = "Context";

    fn to_lua_string<'a>(&self, lua: &'a Lua) -> LuaResult<LuaString<'a>> {
        lua.create_string(format!("Context {{ state = {:?} }}", self.get_state()))
    }

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method_mut("connect", |_, this, server: Option<String>| {
            let server = server.as_deref();
            this.connect(server, pulse::context::FlagSet::NOAUTOSPAWN, None)
                .into_lua_err()?;

            Ok(())
        });

        reg.add_method_mut(
            "set_state_callback",
            |_, this, f: Option<LuaOwnedFunction>| {
                if let Some(f) = f {
                    this.set_state_callback(Some(Box::new(move || {
                        catch_lua_errors::<_, ()>(f.to_ref(), ());
                    })));
                } else {
                    this.set_state_callback(None);
                }

                Ok(())
            },
        );

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

        reg.add_method_mut(
            "set_subscribe_callback",
            |_, this, f: Option<LuaOwnedFunction>| {
                if let Some(f) = f {
                    this.set_subscribe_callback(Some(Box::new(
                        move |facility, operation, index| {
                            let facility = facility.map(|value| value as i32);
                            let operation = operation.map(|value| value as i32);
                            catch_lua_errors::<_, ()>(f.to_ref(), (facility, operation, index));
                        },
                    )));
                } else {
                    this.set_subscribe_callback(None);
                }

                Ok(())
            },
        );

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
                        if let ListResult::Item(item) = result {
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
                        if let ListResult::Item(item) = result {
                            catch_lua_errors::<_, ()>(f.to_ref(), SinkInfoWrapper(item));
                        }
                    });

                Ok(())
            },
        );

        reg.add_method(
            "set_sink_mute_by_index",
            |_, this, (index, mute, f): (u32, bool, Option<LuaOwnedFunction>)| {
                if let Some(f) = f {
                    this.introspect().set_sink_mute_by_index(
                        index,
                        mute,
                        Some(Box::new(move |success| {
                            catch_lua_errors::<_, ()>(f.to_ref(), success);
                        })),
                    );
                } else {
                    this.introspect().set_sink_mute_by_index(index, mute, None);
                }

                Ok(())
            },
        );

        reg.add_method(
            "set_sink_volume_by_index",
            |_, this, (index, volume, f): (u32, ChannelVolumesWrapper, Option<LuaOwnedFunction>)| {
                if let Some(f) = f {
                    this.introspect().set_sink_volume_by_index(
                        index,
                        &volume.0,
                        Some(Box::new(move |success| {
                            catch_lua_errors::<_, ()>(f.to_ref(), success);
                        })),
                    );
                } else {
                    this.introspect().set_sink_volume_by_index(index, &volume.0, None);
                }

                Ok(())
            },
        );
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(
                |lua, (mainloop, name): (LuaUserDataRef<Mainloop>, String)| {
                    let ctx = Context::new(&*mainloop, &name);
                    if let Some(ctx) = ctx {
                        Ok(Some(lua.create_any_userdata(ctx)?))
                    } else {
                        Ok(None)
                    }
                },
            )?,
        )?;

        Ok(())
    }
}

fn push_volume_constants(lua: &Lua, pulseaudio_table: &LuaTable) -> LuaResult<()> {
    let volume = lua.create_table()?;
    volume.set("NORMAL", Volume::NORMAL.0)?;
    volume.set("MUTED", Volume::MUTED.0)?;
    volume.set("MAX", Volume::MAX.0)?;
    volume.set("INVALID", Volume::INVALID.0)?;
    pulseaudio_table.set("Volume", volume)?;

    Ok(())
}

pub fn push_api(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
    let pulseaudio_table = lua.create_table()?;

    push_enums(lua, &pulseaudio_table)?;
    push_volume_constants(lua, &pulseaudio_table)?;
    Mainloop::push_lua(lua, &pulseaudio_table)?;
    Context::push_lua(lua, &pulseaudio_table)?;

    table.set("pulseaudio", pulseaudio_table)?;

    Ok(())
}
