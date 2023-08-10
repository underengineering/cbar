use gtk::{glib, prelude::*, Application, ApplicationWindow};
use mlua::{prelude::*, Variadic};

fn add_widget_methods<T: glib::IsA<gtk::Widget>>(reg: &mut LuaUserDataRegistry<'_, T>) {
    reg.add_method("upcast", |lua, this, ()| {
        lua.create_any_userdata(this.clone().upcast::<gtk::Widget>())
    });

    reg.add_method("set_css_class", |lua, this, class: String| {
        this.add_css_class(&class);
        Ok(())
    });

    reg.add_method("set_css_classes", |lua, this, classes: Variadic<String>| {
        this.set_css_classes(&classes.iter().map(String::as_str).collect::<Vec<_>>());
        Ok(())
    });
}

fn add_enums(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    let orientation = lua.create_table()?;
    orientation.set("Horizontal", 0)?;
    orientation.set("Vertical", 1)?;
    gtk_table.set("Orientation", orientation)?;

    // gtk_table.set("PRIORITY_LOW", glib::ffi::G_PRIORITY_LOW)?;
    // gtk_table.set("PRIORITY_DEFAULT", glib::ffi::G_PRIORITY_DEFAULT)?;
    // gtk_table.set("PRIORITY_DEFAULT_IDLE", glib::ffi::G_PRIORITY_DEFAULT_IDLE)?;
    // gtk_table.set("PRIORITY_HIGH", glib::ffi::G_PRIORITY_HIGH)?;
    // gtk_table.set("PRIORITY_HIGH_IDLE", glib::ffi::G_PRIORITY_HIGH_IDLE)?;

    Ok(())
}

fn add_global_functions(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    gtk_table.set(
        "style_context_add_provider",
        lua.create_function(|_, provider: LuaUserDataRef<gtk::CssProvider>| {
            gtk::style_context_add_provider_for_display(
                &gtk::gdk::Display::default().expect("Could not connect to the display"),
                &*provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            Ok(())
        })?,
    )?;

    Ok(())
}

fn add_application_api(lua: &Lua) -> LuaResult<()> {
    lua.register_userdata_type::<Application>(|reg| {
        reg.add_method("connect_activate", |_, this, f: LuaOwnedFunction| {
            this.connect_activate(move |_| {
                f.call::<_, ()>(()).unwrap();
            });
            Ok(())
        });

        reg.add_method("connect_startup", |_, this, f: LuaOwnedFunction| {
            this.connect_startup(move |_| {
                f.call::<_, ()>(()).unwrap();
            });
            Ok(())
        });

        reg.add_method("connect_shutdown", |_, this, f: LuaOwnedFunction| {
            this.connect_shutdown(move |_| {
                f.call::<_, ()>(()).unwrap();
            });
            Ok(())
        });

        reg.add_method("run", |_, this, ()| {
            this.run();
            Ok(())
        });
    })?;

    Ok(())
}

fn add_application_window_api(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<ApplicationWindow>(|reg| {
        reg.add_method("set_title", |_, this, title: String| {
            this.set_title(Some(&title));
            Ok(())
        });

        reg.add_method(
            "set_child",
            |_, this, child: LuaUserDataRef<gtk::Widget>| {
                this.set_child(Some(&*child));
                Ok(())
            },
        );

        reg.add_method("present", |_, this, ()| {
            this.present();
            Ok(())
        });
    })?;
    let window = lua.create_table()?;
    window.set(
        "new",
        lua.create_function(|lua, app: LuaUserDataRef<Application>| {
            let window = ApplicationWindow::new(&*app);
            lua.create_any_userdata(window)
        })?,
    )?;
    gtk_table.set("ApplicationWindow", window)?;

    Ok(())
}

fn add_button_api(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<gtk::Button>(|reg| {
        reg.add_method("connect_clicked", |_, this, f: LuaOwnedFunction| {
            this.connect_clicked(move |_| f.call::<_, ()>(()).unwrap());
            Ok(())
        });

        reg.add_method("set_label", |_, this, label: String| {
            this.set_label(&label);
            Ok(())
        });

        add_widget_methods(reg);
    })?;
    let button = lua.create_table()?;
    button.set(
        "new",
        lua.create_function(|lua, ()| {
            let button = gtk::Button::new();
            lua.create_any_userdata(button)
        })?,
    )?;
    gtk_table.set("Button", button)?;

    Ok(())
}

fn add_label_api(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<gtk::Label>(|reg| {
        reg.add_method("set_label", |_, this, str: String| {
            this.set_text(&str);
            Ok(())
        });

        reg.add_method("set_markup", |_, this, markup: String| {
            this.set_markup(&markup);
            Ok(())
        });

        add_widget_methods(reg);
    })?;
    let button = lua.create_table()?;
    button.set(
        "new",
        lua.create_function(|lua, str: Option<String>| {
            let button = gtk::Label::new(str.as_deref());
            lua.create_any_userdata(button)
        })?,
    )?;
    gtk_table.set("Label", button)?;

    Ok(())
}

fn add_box_api(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<gtk::Box>(|reg| {
        reg.add_method("append", |_, this, child: LuaUserDataRef<gtk::Widget>| {
            this.append(&*child);
            Ok(())
        });

        add_widget_methods(reg);
    })?;
    let gbox = lua.create_table()?;
    gbox.set(
        "new",
        lua.create_function(|lua, (orientation, spacing): (i32, Option<i32>)| {
            let orientation = match orientation {
                0 => gtk::Orientation::Horizontal,
                1 => gtk::Orientation::Vertical,
                _ => panic!("Invalid orientation"),
            };

            let button = gtk::Box::new(orientation, spacing.unwrap_or(0));
            lua.create_any_userdata(button)
        })?,
    )?;
    gtk_table.set("Box", gbox)?;

    Ok(())
}

fn add_css_provider(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<gtk::CssProvider>(|reg| {
        reg.add_method("load_from_data", |_, this, data: String| {
            this.load_from_data(&data);
            Ok(())
        });

        reg.add_method("load_from_path", |_, this, path: String| {
            this.load_from_path(path);
            Ok(())
        });
    })?;
    let gbox = lua.create_table()?;
    gbox.set(
        "new",
        lua.create_function(|lua, ()| {
            let provider = gtk::CssProvider::new();
            lua.create_any_userdata(provider)
        })?,
    )?;
    gtk_table.set("CssProvider", gbox)?;

    Ok(())
}

fn add_context_api(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<glib::MainContext>(|reg| {
        reg.add_method("spawn_local", |_, this, f: LuaOwnedFunction| {
            this.spawn_local(async move { f.call_async::<_, ()>(()).await.unwrap() });
            Ok(())
        });
    })?;
    let ctx = lua.create_table()?;
    ctx.set(
        "default",
        lua.create_function(|lua, ()| {
            let ctx = glib::MainContext::default();
            lua.create_any_userdata(ctx)
        })?,
    )?;
    gtk_table.set("MainContext", ctx)?;

    Ok(())
}

pub fn add_api(lua: &Lua) -> LuaResult<LuaTable> {
    let gtk_table = lua.create_table()?;

    add_enums(lua, &gtk_table)?;
    add_global_functions(lua, &gtk_table)?;
    add_application_api(lua)?;
    add_application_window_api(lua, &gtk_table)?;
    add_label_api(lua, &gtk_table)?;
    add_button_api(lua, &gtk_table)?;
    add_box_api(lua, &gtk_table)?;
    add_css_provider(lua, &gtk_table)?;
    add_context_api(lua, &gtk_table)?;

    Ok(gtk_table)
}
