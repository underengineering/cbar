use gtk::Application;
use mlua::prelude::*;

const APP_ID: &str = "org.gtk_rs.HelloWorld1";

mod hyprland;
mod luaapi;
mod system_info;

#[tokio::main]
async fn main() -> LuaResult<()> {
    let lua = Lua::new();
    let globals = lua.globals();
    let gtk_table = luaapi::gtk::add_api(&lua)?;
    let utils_table = luaapi::utils::add_api(&lua)?;
    let hyprland_table = luaapi::hyprland::add_api(&lua)?;
    let sysinfo_table = luaapi::sysinfo::add_api(&lua)?;
    globals.set("utils", utils_table)?;
    globals.set("hyprland", hyprland_table)?;
    globals.set("sysinfo", sysinfo_table)?;

    let app = Application::builder().application_id(APP_ID).build();

    gtk_table.set("app", lua.create_any_userdata(app)?)?;
    globals.set("gtk", gtk_table)?;

    lua.load(
        r#"
        print(utils.lookup_icon("org.wezfurlong.wezterm"))

        print("is on AC:", sysinfo.battery.is_on_ac())

        local info = sysinfo.battery.get_batteries()
        print("total capacity:", info.total_capacity)
        print("remaining time:", info.remaining_time.secs)
        print'Batteries:'
        for name, info in pairs(info.info) do
            print(name, "capacity:", info.capacity, "remaining_time:", info.remaining_time.secs, "status:", info.status)
        end

        local ctx = gtk.MainContext.default()
        ctx:spawn_local(function()
            print'running in async ctx'
            utils.sleep(1.5)
            print'after 1.5s'
        end)

        ctx:spawn_local(function()
            print'ipc'
            local resp = hyprland.ipc_request("workspaces")
            print("resp", resp)
            for k, v in pairs(resp) do
                print(k, v)
                for i, j in pairs(v) do
                    print(i, j)
                end
            end
        end)

        ctx:spawn_local(function()
            print'running event loop'

            local event_loop = hyprland.EventLoop.connect()
            print'connected to event loop'

            local event_receiver = event_loop:receiver()
            ctx:spawn_local(function()
                while true do
                    local ev = event_receiver:recv()
                    print("event", ev)
                    for k, v in pairs(ev) do
                        print(k, v)
                    end
                end
            end)
        
            event_loop:run()
        end)

        gtk.app:connect_activate(function()
            print'activate'

            local win = gtk.ApplicationWindow.new(gtk.app)
            win:set_title("Window title")

            local box = gtk.Box.new(gtk.Orientation.Horizontal, 0)

            local btn = gtk.Button.new()
            btn:set_label("test")

            local lbl = gtk.Label.new()
            lbl:set_markup("<small>adsadasd</small>")

            box:append(btn:upcast())
            box:append(lbl:upcast())

            btn:connect_clicked(function()
                local text = gtk.Label.new("STOP PRESSING PLS")
                box:append(text:upcast())
            end)

            win:set_child(box:upcast())

            win:present()
        end)

        gtk.app:run()
    "#,
    )
    .exec()?;

    Ok(())
}
