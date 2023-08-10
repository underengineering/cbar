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
    luaapi::utils::add_api(&lua)?;

    let app = Application::builder().application_id(APP_ID).build();

    gtk_table.set("app", lua.create_any_userdata(app)?)?;
    globals.set("gtk", gtk_table)?;

    lua.load(
        r#"
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
