use gtk::{glib, prelude::*, Application, ApplicationWindow};
use mlua::prelude::*;

use super::enums;

macro_rules! push_enum {
    ($tbl:ident, $ns:ident, $name:ident, [$($variant:ident),+]) => {
        $($tbl.set(stringify!($variant), enums::$name($ns::$name::$variant))?;)+
    };
}

fn add_widget_methods<T: glib::IsA<gtk::Widget>>(reg: &mut LuaUserDataRegistry<'_, T>) {
    reg.add_method("upcast", |lua, this, ()| {
        lua.create_any_userdata(this.clone().upcast::<gtk::Widget>())
    });

    reg.add_method("set_css_class", |_lua, this, class: String| {
        this.add_css_class(&class);
        Ok(())
    });

    reg.add_method("set_css_classes", |_lua, this, classes: Vec<String>| {
        this.set_css_classes(&classes.iter().map(String::as_str).collect::<Vec<_>>());
        Ok(())
    });

    reg.add_method("add_css_class", |_lua, this, class: String| {
        this.add_css_class(&class);
        Ok(())
    });

    reg.add_method("remove_css_class", |_lua, this, class: String| {
        this.remove_css_class(&class);
        Ok(())
    });

    reg.add_method("set_valign", |_lua, this, align: enums::Align| {
        this.set_valign(align.0);
        Ok(())
    });

    reg.add_method("set_halign", |_lua, this, align: enums::Align| {
        this.set_halign(align.0);
        Ok(())
    });

    reg.add_method(
        "set_size_request",
        |_, this, (width, height): (i32, i32)| {
            this.set_size_request(width, height);
            Ok(())
        },
    );

    reg.add_method("allocated_width", |_, this, ()| Ok(this.allocated_width()));
    reg.add_method(
        "allocated_height",
        |_, this, ()| Ok(this.allocated_height()),
    );

    reg.add_method(
        "set_layout_manager",
        |_, this, layout_manager: Option<LuaOwnedAnyUserData>| {
            let layout_manager = match layout_manager {
                Some(ud) => Some(ud.take::<gtk::LayoutManager>()?),
                None => None,
            };

            this.set_layout_manager(layout_manager);
            Ok(())
        },
    );
}

// fn add_layout_manager_methods<T: glib::IsA<gtk::LayoutManager>>(
//     reg: &mut LuaUserDataRegistry<'_, T>,
// ) {
//     reg.add_method("upcast", |lua, this, ()| {
//         lua.create_any_userdata(this.clone().upcast::<gtk::LayoutManager>())
//     });
// }

fn add_enums(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    let orientation = lua.create_table()?;
    push_enum!(orientation, gtk, Orientation, [Horizontal, Vertical]);
    gtk_table.set("Orientation", orientation)?;

    let align = lua.create_table()?;
    push_enum!(align, gtk, Align, [Fill, Start, End, Center, Baseline]);
    gtk_table.set("Align", align)?;

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
            this.run_with_args(&[""]);
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

        add_widget_methods(reg);
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

        reg.add_method("remove", |_, this, child: LuaUserDataRef<gtk::Widget>| {
            this.remove(&*child);
            Ok(())
        });

        add_widget_methods(reg);
    })?;
    let gbox = lua.create_table()?;
    gbox.set(
        "new",
        lua.create_function(
            |lua, (orientation, spacing): (enums::Orientation, Option<i32>)| {
                let gbox = gtk::Box::new(orientation.0, spacing.unwrap_or(0));
                lua.create_any_userdata(gbox)
            },
        )?,
    )?;
    gtk_table.set("Box", gbox)?;

    Ok(())
}

fn add_center_box_api(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<gtk::CenterBox>(|reg| {
        reg.add_method(
            "set_start_widget",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                if let Some(udref) = child {
                    this.set_start_widget(Some(&*udref));
                } else {
                    this.set_start_widget(None::<&gtk::Widget>);
                }

                Ok(())
            },
        );

        reg.add_method(
            "set_center_widget",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                if let Some(udref) = child {
                    this.set_center_widget(Some(&*udref));
                } else {
                    this.set_center_widget(None::<&gtk::Widget>);
                }

                Ok(())
            },
        );

        reg.add_method(
            "set_end_widget",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                if let Some(udref) = child {
                    this.set_end_widget(Some(&*udref));
                } else {
                    this.set_end_widget(None::<&gtk::Widget>);
                }

                Ok(())
            },
        );

        add_widget_methods(reg);
    })?;
    let center_box = lua.create_table()?;
    center_box.set(
        "new",
        lua.create_function(|lua, ()| {
            let center_box = gtk::CenterBox::new();
            lua.create_any_userdata(center_box)
        })?,
    )?;
    gtk_table.set("CenterBox", center_box)?;

    Ok(())
}

fn add_image_api(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    lua.register_userdata_type::<gtk::Image>(|reg| {
        reg.add_method("set_pixel_size", |_, this, pixel_size: i32| {
            this.set_pixel_size(pixel_size);
            Ok(())
        });

        reg.add_method("set_from_file", |_, this, path: Option<String>| {
            this.set_from_file(path);
            Ok(())
        });

        reg.add_method(
            "set_from_icon_name",
            |_, this, icon_name: Option<String>| {
                this.set_from_icon_name(icon_name.as_deref());
                Ok(())
            },
        );

        add_widget_methods(reg);
    })?;
    let image = lua.create_table()?;
    image.set(
        "new",
        lua.create_function(|lua, ()| {
            let image = gtk::Image::new();
            lua.create_any_userdata(image)
        })?,
    )?;
    image.set(
        "from_file",
        lua.create_function(|lua, path: String| {
            let image = gtk::Image::from_file(path);
            lua.create_any_userdata(image)
        })?,
    )?;
    image.set(
        "from_icon_name",
        lua.create_function(|lua, icon_name: String| {
            let image = gtk::Image::from_icon_name(&icon_name);
            lua.create_any_userdata(image)
        })?,
    )?;
    gtk_table.set("Image", image)?;

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

fn add_layer_shell_api(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    let layer_shell = lua.create_table()?;

    let layer = lua.create_table()?;
    push_enum!(
        layer,
        gtk4_layer_shell,
        Layer,
        [Background, Bottom, Top, Overlay]
    );
    layer_shell.set("Layer", layer)?;

    let edge = lua.create_table()?;
    push_enum!(edge, gtk4_layer_shell, Edge, [Left, Right, Top, Bottom]);
    layer_shell.set("Edge", edge)?;

    layer_shell.set(
        "init_for_window",
        lua.create_function(|_, window: LuaUserDataRef<ApplicationWindow>| {
            gtk4_layer_shell::init_for_window(&*window);
            Ok(())
        })?,
    )?;
    layer_shell.set(
        "set_layer",
        lua.create_function(
            |_, (window, layer): (LuaUserDataRef<ApplicationWindow>, enums::Layer)| {
                gtk4_layer_shell::set_layer(&*window, layer.0);
                Ok(())
            },
        )?,
    )?;
    layer_shell.set(
        "auto_exclusive_zone_enable",
        lua.create_function(|_, window: LuaUserDataRef<ApplicationWindow>| {
            gtk4_layer_shell::auto_exclusive_zone_enable(&*window);
            Ok(())
        })?,
    )?;
    layer_shell.set(
        "set_exclusive_zone",
        lua.create_function(
            |_, (window, exclusive_zone): (LuaUserDataRef<ApplicationWindow>, i32)| {
                gtk4_layer_shell::set_exclusive_zone(&*window, exclusive_zone);
                Ok(())
            },
        )?,
    )?;
    layer_shell.set(
        "set_margin",
        lua.create_function(
            |_, (window, edge, margin_size): (LuaUserDataRef<ApplicationWindow>, enums::Edge, i32)| {
                gtk4_layer_shell::set_margin(&*window, edge.0, margin_size);
                Ok(())
            },
        )?,
    )?;
    layer_shell.set(
        "set_anchor",
        lua.create_function(
            |_,
             (window, edge, anchor_to_edge): (
                LuaUserDataRef<ApplicationWindow>,
                enums::Edge,
                bool,
            )| {
                gtk4_layer_shell::set_anchor(&*window, edge.0, anchor_to_edge);
                Ok(())
            },
        )?,
    )?;

    gtk_table.set("layer_shell", layer_shell)?;

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
    add_center_box_api(lua, &gtk_table)?;
    add_image_api(lua, &gtk_table)?;
    add_css_provider(lua, &gtk_table)?;
    add_context_api(lua, &gtk_table)?;
    add_layer_shell_api(lua, &gtk_table)?;

    Ok(gtk_table)
}
