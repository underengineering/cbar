use mlua::prelude::*;
use sysinfo::{
    Cpu, CpuExt, CpuRefreshKind, NetworkExt, Pid, ProcessRefreshKind, System, SystemExt,
};

use crate::system_info::battery;

fn add_system_api(lua: &Lua, sysinfo_table: &LuaTable) -> LuaResult<()> {
    let cpu_refresh_kind = lua.create_table()?;
    cpu_refresh_kind.set(
        "new",
        lua.create_function(|lua, specifics: LuaTable| {
            let mut kind = CpuRefreshKind::new();
            if specifics
                .get::<_, Option<bool>>("frequency")?
                .unwrap_or(false)
            {
                kind = kind.with_frequency()
            }

            if specifics
                .get::<_, Option<bool>>("cpu_usage")?
                .unwrap_or(false)
            {
                kind = kind.with_cpu_usage()
            }

            lua.create_any_userdata(kind)
        })?,
    )?;
    sysinfo_table.set("CpuRefreshKind", cpu_refresh_kind)?;

    let process_refresh_kind = lua.create_table()?;
    process_refresh_kind.set(
        "new",
        lua.create_function(|lua, specifics: LuaTable| {
            let mut kind = ProcessRefreshKind::new();
            if specifics.get::<_, Option<bool>>("cpu")?.unwrap_or(false) {
                kind = kind.with_cpu();
            }

            if specifics
                .get::<_, Option<bool>>("disk_usage")?
                .unwrap_or(false)
            {
                kind = kind.with_disk_usage();
            }

            if specifics.get::<_, Option<bool>>("user")?.unwrap_or(false) {
                kind = kind.with_user();
            }

            lua.create_any_userdata(kind)
        })?,
    )?;
    sysinfo_table.set("ProcessRefreshKind", process_refresh_kind)?;

    lua.register_userdata_type::<System>(|reg| {
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
            |_, this, kind: LuaOwnedAnyUserData| {
                let kind = kind.take::<CpuRefreshKind>()?;
                this.refresh_cpu_specifics(kind);
                Ok(())
            },
        );

        reg.add_method_mut("refresh_process", |_, this, pid: usize| {
            this.refresh_process(Pid::from(pid));
            Ok(())
        });

        reg.add_method_mut(
            "refresh_processes_specifics",
            |_, this, kind: LuaOwnedAnyUserData| {
                let kind = kind.take::<ProcessRefreshKind>()?;
                this.refresh_processes_specifics(kind);
                Ok(())
            },
        );

        reg.add_method_mut(
            "refresh_process_specifics",
            |_, this, (pid, kind): (usize, LuaOwnedAnyUserData)| {
                let kind = kind.take::<ProcessRefreshKind>()?;
                this.refresh_process_specifics(Pid::from(pid), kind);
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
            let cpus_table = lua.create_table()?;
            for (i, cpu) in cpus.iter().enumerate() {
                let cpu_table = lua.create_table()?;
                push_cpu(cpu, &cpu_table)?;
                cpus_table.set(i, cpu_table)?;
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
                let data_table = lua.create_table()?;
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
    })?;
    let system = lua.create_table()?;
    system.set(
        "new_all",
        lua.create_function(|lua, ()| {
            let system = System::new_all();
            lua.create_any_userdata(system)
        })?,
    )?;

    Ok(())
}

fn add_battery_api(lua: &Lua, sysinfo_table: &LuaTable) -> LuaResult<()> {
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

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let sysinfo_table = lua.create_table()?;

    add_system_api(lua, &sysinfo_table)?;
    add_battery_api(lua, &sysinfo_table)?;

    Ok(sysinfo_table)
}
