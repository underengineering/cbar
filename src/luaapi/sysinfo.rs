use mlua::prelude::*;
use sysinfo::{Cpu, CpuExt, NetworkExt, Pid, System, SystemExt};

use super::wrappers::RefreshKindWrapper;
use crate::{
    luaapi::wrappers::{CpuRefreshKindWrapper, ProcessRefreshKindWrapper},
    system_info::battery,
    traits::LuaApi,
};

impl LuaApi for System {
    const CLASS_NAME: &'static str = "System";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        // Refresh methods
        reg.add_method_mut("refresh_all", |_, this, ()| {
            this.refresh_all();
            Ok(())
        });

        reg.add_method_mut("refresh_system", |_, this, ()| {
            this.refresh_system();
            Ok(())
        });

        reg.add_method_mut("refresh_memory", |_, this, ()| {
            this.refresh_memory();
            Ok(())
        });

        reg.add_method_mut("refresh_cpu", |_, this, ()| {
            this.refresh_cpu();
            Ok(())
        });

        reg.add_method_mut(
            "refresh_cpu_specifics",
            |_, this, kind: CpuRefreshKindWrapper| {
                this.refresh_cpu_specifics(kind.0);
                Ok(())
            },
        );

        reg.add_method_mut("refresh_process", |_, this, pid: usize| {
            this.refresh_process(Pid::from(pid));
            Ok(())
        });

        reg.add_method_mut(
            "refresh_processes_specifics",
            |_, this, kind: ProcessRefreshKindWrapper| {
                this.refresh_processes_specifics(kind.0);
                Ok(())
            },
        );

        reg.add_method_mut(
            "refresh_process_specifics",
            |_, this, (pid, kind): (usize, ProcessRefreshKindWrapper)| {
                this.refresh_process_specifics(Pid::from(pid), kind.0);
                Ok(())
            },
        );

        reg.add_method_mut("refresh_disks", |_, this, ()| {
            this.refresh_disks();
            Ok(())
        });

        reg.add_method_mut("refresh_disks_list", |_, this, ()| {
            this.refresh_disks_list();
            Ok(())
        });

        reg.add_method_mut("refresh_users_list", |_, this, ()| {
            this.refresh_users_list();
            Ok(())
        });

        reg.add_method_mut("refresh_networks", |_, this, ()| {
            this.refresh_networks();
            Ok(())
        });

        reg.add_method_mut("refresh_networks_list", |_, this, ()| {
            this.refresh_networks_list();
            Ok(())
        });

        reg.add_method_mut("refresh_components", |_, this, ()| {
            this.refresh_components();
            Ok(())
        });

        reg.add_method_mut("refresh_components_list", |_, this, ()| {
            this.refresh_components_list();
            Ok(())
        });

        // Getters
        fn push_cpu(cpu: &Cpu, table: &LuaTable) -> LuaResult<()> {
            table.set("name", cpu.name())?;
            table.set("frequency", cpu.frequency())?;
            table.set("vendor_id", cpu.vendor_id())?;
            table.set("brand", cpu.brand())?;
            table.set("cpu_usage", cpu.cpu_usage())?;
            Ok(())
        }

        reg.add_method("cpus", |lua, this, ()| {
            let cpus = this.cpus();
            let cpus_table = lua.create_table_with_capacity(cpus.len(), 0)?;
            for (i, cpu) in cpus.iter().enumerate() {
                let cpu_table = lua.create_table_with_capacity(0, 5)?;
                push_cpu(cpu, &cpu_table)?;
                cpus_table.set(i + 1, cpu_table)?;
            }

            Ok(cpus_table)
        });

        reg.add_method("global_cpu_info", |lua, this, ()| {
            let cpu = this.global_cpu_info();
            let cpu_table = lua.create_table()?;
            push_cpu(cpu, &cpu_table)?;

            Ok(cpu_table)
        });

        reg.add_method("total_memory", |_, this, ()| {
            Ok(LuaValue::Integer(this.total_memory() as i64))
        });

        reg.add_method("available_memory", |_, this, ()| {
            Ok(LuaValue::Integer(this.available_memory() as i64))
        });

        reg.add_method("used_memory", |_, this, ()| {
            Ok(LuaValue::Integer(this.used_memory() as i64))
        });

        reg.add_method("free_memory", |_, this, ()| {
            Ok(LuaValue::Integer(this.free_memory() as i64))
        });

        reg.add_method("total_swap", |_, this, ()| {
            Ok(LuaValue::Integer(this.total_swap() as i64))
        });

        reg.add_method("free_swap", |_, this, ()| {
            Ok(LuaValue::Integer(this.free_swap() as i64))
        });

        reg.add_method("used_swap", |_, this, ()| {
            Ok(LuaValue::Integer(this.used_swap() as i64))
        });

        reg.add_method("networks", |lua, this, ()| {
            let networks_table = lua.create_table()?;
            for (iface, data) in this.networks() {
                let data_table = lua.create_table_with_capacity(0, 13)?;
                macro_rules! copy_fields {
                    ($name:ident) => {
                        data_table.set(stringify!($name), data.$name())?;
                    };
                }

                copy_fields!(received);
                copy_fields!(total_received);
                copy_fields!(transmitted);
                copy_fields!(total_transmitted);
                copy_fields!(packets_received);
                copy_fields!(total_packets_received);
                copy_fields!(packets_transmitted);
                copy_fields!(total_packets_transmitted);
                copy_fields!(errors_on_received);
                copy_fields!(total_errors_on_received);
                copy_fields!(errors_on_transmitted);
                copy_fields!(total_errors_on_transmitted);
                data_table.set("mac_address", data.mac_address().0)?;

                networks_table.set(iface.as_str(), data_table)?;
            }

            Ok(networks_table)
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new_all",
            lua.create_function(|lua, ()| {
                let system = System::new_all();
                lua.create_any_userdata(system)
            })?,
        )?;
        table.set(
            "new_with_specifics",
            lua.create_function(|lua, kind: RefreshKindWrapper| {
                let system = System::new_with_specifics(kind.0);
                lua.create_any_userdata(system)
            })?,
        )?;

        Ok(())
    }
}

fn push_battery_api(lua: &Lua, sysinfo_table: &LuaTable) -> LuaResult<()> {
    let battery_table = lua.create_table()?;
    battery_table.set(
        "is_on_ac",
        lua.create_function(|_, ()| Ok(battery::is_on_ac()))?,
    )?;
    battery_table.set(
        "get_batteries",
        lua.create_function(|lua, ()| {
            let batteries = battery::get_batteries();
            lua.to_value(&batteries)
        })?,
    )?;

    sysinfo_table.set("battery", battery_table)?;

    Ok(())
}

pub fn push_api(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
    let sysinfo_table = lua.create_table()?;

    System::push_lua(lua, &sysinfo_table)?;
    push_battery_api(lua, &sysinfo_table)?;

    table.set("sysinfo", sysinfo_table)?;

    Ok(())
}
