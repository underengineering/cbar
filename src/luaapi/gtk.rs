use gtk::{
    cairo,
    gio::Icon,
    glib::{self, MainContext},
    pango,
    prelude::*,
    Application, ApplicationWindow, Box, Button, CenterBox, CheckButton, CssProvider, DrawingArea,
    Entry, EventControllerFocus, EventControllerKey, EventControllerMotion, EventControllerScroll,
    Grid, Image, Label, Overlay, Revealer, Scale, Settings,
};
use mlua::prelude::*;
use paste::paste;

use super::{
    enums,
    wrappers::{
        ApplicationFlagsWrapper, ContextWrapper, EventControllerScrollFlagsWrapper, GStringWrapper,
        ModifierTypeWrapper,
    },
};
use crate::utils::{catch_lua_errors, catch_lua_errors_async};
use crate::{macros::register_signals, traits::LuaApi};

macro_rules! push_enum {
    ($lua:ident, $tbl:ident, $ns:ident, $name:ident, [$($variant:ident),+]) => {
        let enum_table = $lua.create_table()?;
        $(enum_table.set(stringify!($variant), enums::$name($ns::$name::$variant))?;)+
        $tbl.set(stringify!($name), enum_table)?;
    };
}

fn add_widget_methods<T: glib::IsA<gtk::Widget>>(reg: &mut LuaUserDataRegistry<'_, T>) {
    reg.add_method("upcast", |lua, this, ()| {
        lua.create_any_userdata(this.clone().upcast::<gtk::Widget>())
    });

    reg.add_method(
        "add_controller",
        |_, this, controller: LuaUserDataRef<gtk::EventController>| {
            this.add_controller(controller.clone());
            Ok(())
        },
    );

    reg.add_method(
        "remove_controller",
        |_, this, controller: LuaUserDataRef<gtk::EventController>| {
            this.remove_controller(&*controller);
            Ok(())
        },
    );

    reg.add_method("set_visible", |_, this, visible: bool| {
        this.set_visible(visible);
        Ok(())
    });

    reg.add_method("get_visible", |_, this, ()| Ok(this.get_visible()));

    reg.add_method("set_css_class", |_, this, class: String| {
        this.add_css_class(&class);
        Ok(())
    });

    reg.add_method("set_css_classes", |_, this, classes: Vec<String>| {
        this.set_css_classes(&classes.iter().map(String::as_str).collect::<Vec<_>>());
        Ok(())
    });

    reg.add_method("add_css_class", |_, this, class: String| {
        this.add_css_class(&class);
        Ok(())
    });

    reg.add_method("remove_css_class", |_, this, class: String| {
        this.remove_css_class(&class);
        Ok(())
    });

    reg.add_method("set_sensitive", |_, this, sensitive: bool| {
        this.set_sensitive(sensitive);
        Ok(())
    });

    reg.add_method("is_sensitive", |_, this, ()| Ok(this.is_sensitive()));

    reg.add_method("set_valign", |_, this, align: enums::Align| {
        this.set_valign(align.0);
        Ok(())
    });

    reg.add_method("set_halign", |_, this, align: enums::Align| {
        this.set_halign(align.0);
        Ok(())
    });

    reg.add_method("set_vexpand", |_, this, expand: bool| {
        this.set_vexpand(expand);
        Ok(())
    });

    reg.add_method("set_hexpand", |_, this, expand: bool| {
        this.set_hexpand(expand);
        Ok(())
    });

    reg.add_method(
        "set_size_request",
        |_, this, (width, height): (i32, i32)| {
            this.set_size_request(width, height);
            Ok(())
        },
    );

    reg.add_method("queue_draw", |_, this, ()| {
        this.queue_draw();
        Ok(())
    });

    reg.add_method("grab_focus", |_, this, ()| Ok(this.grab_focus()));

    reg.add_method("settings", |lua, this, ()| {
        let settings = this.settings();
        lua.create_any_userdata(settings)
    });

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

fn push_enums(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    push_enum!(lua, gtk_table, gtk, Orientation, [Horizontal, Vertical]);

    push_enum!(
        lua,
        gtk_table,
        gtk,
        Align,
        [Fill, Start, End, Center, Baseline]
    );

    push_enum!(
        lua,
        gtk_table,
        pango,
        EllipsizeMode,
        [None, Start, Middle, End]
    );

    push_enum!(
        lua,
        gtk_table,
        cairo,
        Operator,
        [
            Clear,
            Source,
            Over,
            In,
            Out,
            Atop,
            Dest,
            DestOver,
            DestIn,
            DestOut,
            DestAtop,
            Xor,
            Add,
            Saturate,
            Multiply,
            Screen,
            Overlay,
            Darken,
            Lighten,
            ColorDodge,
            ColorBurn,
            HardLight,
            SoftLight,
            Difference,
            Exclusion,
            HslHue,
            HslSaturation,
            HslColor,
            HslLuminosity
        ]
    );

    push_enum!(
        lua,
        gtk_table,
        gtk,
        RevealerTransitionType,
        [
            None, Crossfade, SlideRight, SlideLeft, SlideUp, SlideDown, SwingRight, SwingLeft,
            SwingUp, SwingDown
        ]
    );

    Ok(())
}

fn push_constants(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    let priority = lua.create_table()?;
    priority.set("HIGH", lua.create_any_userdata(glib::PRIORITY_HIGH)?)?;
    priority.set("DEFAULT", lua.create_any_userdata(glib::PRIORITY_DEFAULT)?)?;
    priority.set(
        "HIGH_IDLE",
        lua.create_any_userdata(glib::PRIORITY_HIGH_IDLE)?,
    )?;
    priority.set(
        "DEFAULT_IDLE",
        lua.create_any_userdata(glib::PRIORITY_DEFAULT_IDLE)?,
    )?;
    priority.set("LOW", lua.create_any_userdata(glib::PRIORITY_LOW)?)?;
    gtk_table.set("Priority", priority)?;

    Ok(())
}

fn add_global_functions(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    gtk_table.set(
        "style_context_add_provider",
        lua.create_function(|_, provider: LuaUserDataRef<CssProvider>| {
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

impl LuaApi for Application {
    const CLASS_NAME: &'static str = "Application";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        register_signals!(reg, [activate, startup, shutdown]);

        reg.add_method("run", |_, this, ()| {
            this.run_with_args(&[""]);
            Ok(())
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(
                |lua, (id, flags): (Option<String>, ApplicationFlagsWrapper)| {
                    let app = Application::new(id, flags.0);
                    lua.create_any_userdata(app)
                },
            )?,
        )?;

        Ok(())
    }
}

impl LuaApi for ApplicationWindow {
    const CLASS_NAME: &'static str = "ApplicationWindow";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("set_title", |_, this, title: Option<String>| {
            this.set_title(title.as_deref());
            Ok(())
        });

        reg.add_method(
            "set_child",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                this.set_child(child.as_deref());
                Ok(())
            },
        );

        reg.add_method("close", |_, this, ()| {
            this.close();
            Ok(())
        });

        reg.add_method("present", |_, this, ()| {
            this.present();
            Ok(())
        });

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, app: LuaUserDataRef<Application>| {
                let window = ApplicationWindow::new(&*app);
                lua.create_any_userdata(window)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for Button {
    const CLASS_NAME: &'static str = "Button";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        register_signals!(reg, [clicked]);

        reg.add_method("set_label", |_, this, label: String| {
            this.set_label(&label);
            Ok(())
        });

        reg.add_method(
            "set_child",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                this.set_child(child.as_deref());
                Ok(())
            },
        );

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let button = Button::new();
                lua.create_any_userdata(button)
            })?,
        )?;
        table.set(
            "with_label",
            lua.create_function(|lua, label: String| {
                let button = Button::with_label(&label);
                lua.create_any_userdata(button)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for CheckButton {
    const CLASS_NAME: &'static str = "CheckButton";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        register_signals!(reg, [toggled]);

        reg.add_method("set_active", |_, this, setting: bool| {
            this.set_active(setting);
            Ok(())
        });

        reg.add_method(
            "set_child",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                this.set_child(child.as_deref());
                Ok(())
            },
        );

        reg.add_method(
            "set_group",
            |_, this, group: Option<LuaUserDataRef<CheckButton>>| {
                this.set_group(group.as_deref());
                Ok(())
            },
        );

        reg.add_method("set_inconsistent", |_, this, inconsistent: bool| {
            this.set_inconsistent(inconsistent);
            Ok(())
        });

        reg.add_method("set_label", |_, this, label: Option<String>| {
            this.set_label(label.as_deref());
            Ok(())
        });

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let button = CheckButton::new();
                lua.create_any_userdata(button)
            })?,
        )?;
        table.set(
            "with_label",
            lua.create_function(|lua, label: String| {
                let check_button = CheckButton::with_label(&label);
                lua.create_any_userdata(check_button)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for Overlay {
    const CLASS_NAME: &'static str = "Overlay";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method(
            "set_child",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                this.set_child(child.as_deref());
                Ok(())
            },
        );

        reg.add_method(
            "add_overlay",
            |_, this, widget: LuaUserDataRef<gtk::Widget>| {
                this.add_overlay(&*widget);
                Ok(())
            },
        );

        reg.add_method(
            "remove_overlay",
            |_, this, widget: LuaUserDataRef<gtk::Widget>| {
                this.remove_overlay(&*widget);
                Ok(())
            },
        );

        reg.add_method(
            "set_measure_overlay",
            |_, this, (widget, measure): (LuaUserDataRef<gtk::Widget>, bool)| {
                this.set_measure_overlay(&*widget, measure);
                Ok(())
            },
        );

        reg.add_method(
            "set_clip_overlay",
            |_, this, (widget, clip_overlay): (LuaUserDataRef<gtk::Widget>, bool)| {
                this.set_clip_overlay(&*widget, clip_overlay);
                Ok(())
            },
        );

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let button = Overlay::new();
                lua.create_any_userdata(button)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for Label {
    const CLASS_NAME: &'static str = "Label";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("set_label", |_, this, str: String| {
            this.set_text(&str);
            Ok(())
        });

        reg.add_method("set_markup", |_, this, markup: String| {
            this.set_markup(&markup);
            Ok(())
        });

        reg.add_method("set_ellipsize", |_, this, mode: enums::EllipsizeMode| {
            this.set_ellipsize(mode.0);
            Ok(())
        });

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, str: Option<String>| {
                let button = Label::new(str.as_deref());
                lua.create_any_userdata(button)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for gtk::EntryBuffer {
    const CLASS_NAME: &'static str = "EntryBuffer";
    const CONSTRUCTIBLE: bool = false;

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method(
            "connect_deleted_text",
            |_, this, (f, after): (LuaOwnedFunction, Option<bool>)| {
                this.connect_local("deleted-text", after.unwrap_or(true), move |values| {
                    if let [_, position, n_chars] = values {
                        let position = position.get::<u32>().unwrap();
                        let n_chars = n_chars.get::<u32>().unwrap();
                        catch_lua_errors::<_, ()>(f.to_ref(), (position, n_chars));
                    }

                    None
                });

                Ok(())
            },
        );

        reg.add_method(
            "connect_inserted_text",
            |_, this, (f, after): (LuaOwnedFunction, Option<bool>)| {
                this.connect_local("inserted-text", after.unwrap_or(true), move |values| {
                    if let [_, position, chars, n_chars] = values {
                        let position = position.get::<u32>().unwrap();
                        let chars = chars.get::<String>().unwrap();
                        let n_chars = n_chars.get::<u32>().unwrap();
                        catch_lua_errors::<_, ()>(f.to_ref(), (position, chars, n_chars));
                    }

                    None
                });

                Ok(())
            },
        );

        reg.add_method("text", |lua, this, ()| lua.create_string(this.text()));

        reg.add_method("set_text", |_, this, chars: String| {
            this.set_text(chars);
            Ok(())
        })
    }
}

impl LuaApi for Entry {
    const CLASS_NAME: &'static str = "Entry";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        register_signals!(reg, [activate]);

        reg.add_method("buffer", |lua, this, ()| {
            lua.create_any_userdata(this.buffer())
        });

        reg.add_method("set_placeholder_text", |_, this, text: Option<String>| {
            this.set_placeholder_text(text.as_deref());
            Ok(())
        });

        reg.add_method("set_alignment", |_, this, xalign: f32| {
            gtk::prelude::EntryExt::set_alignment(this, xalign);
            Ok(())
        });

        reg.add_method("set_visibility", |_, this, visible: bool| {
            this.set_visibility(visible);
            Ok(())
        });

        reg.add_method("set_max_length", |_, this, max: i32| {
            this.set_max_length(max);
            Ok(())
        });

        reg.add_method("grab_focus_without_selecting", |_, this, ()| {
            this.grab_focus_without_selecting();
            Ok(())
        });

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let entry = Entry::new();
                lua.create_any_userdata(entry)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for Box {
    const CLASS_NAME: &'static str = "Box";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("prepend", |_, this, child: LuaUserDataRef<gtk::Widget>| {
            this.prepend(&*child);
            Ok(())
        });

        reg.add_method("append", |_, this, child: LuaUserDataRef<gtk::Widget>| {
            this.append(&*child);
            Ok(())
        });

        reg.add_method(
            "reorder_child_after",
            |_,
             this,
             (child, sibling): (
                LuaUserDataRef<gtk::Widget>,
                Option<LuaUserDataRef<gtk::Widget>>,
            )| {
                this.reorder_child_after(&*child, sibling.as_deref());
                Ok(())
            },
        );

        reg.add_method("remove", |_, this, child: LuaUserDataRef<gtk::Widget>| {
            this.remove(&*child);
            Ok(())
        });

        reg.add_method("remove_all", |_, this, ()| {
            let mut child_opt = this.first_child();
            while let Some(child) = child_opt {
                this.remove(&child);
                child_opt = child.next_sibling();
                if child_opt.is_none() {
                    break;
                }
            }

            Ok(())
        });

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(
                |lua, (orientation, spacing): (enums::Orientation, Option<i32>)| {
                    let gbox = Box::new(orientation.0, spacing.unwrap_or(0));
                    lua.create_any_userdata(gbox)
                },
            )?,
        )?;

        Ok(())
    }
}

impl LuaApi for Grid {
    const CLASS_NAME: &'static str = "Grid";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method(
            "attach",
            |_,
             this,
             (child, column, row, width, height): (
                LuaUserDataRef<gtk::Widget>,
                i32,
                i32,
                Option<i32>,
                Option<i32>,
            )| {
                this.attach(
                    &*child,
                    column,
                    row,
                    width.unwrap_or(1),
                    height.unwrap_or(1),
                );
                Ok(())
            },
        );

        reg.add_method("remove", |_, this, child: LuaUserDataRef<gtk::Widget>| {
            this.remove(&*child);
            Ok(())
        });

        reg.add_method("remove_all", |_, this, ()| {
            while let Some(child) = this.first_child() {
                this.remove(&child);
            }

            Ok(())
        });

        reg.add_method("set_column_spacing", |_, this, spacing: u32| {
            this.set_column_spacing(spacing);
            Ok(())
        });

        reg.add_method("set_column_homogeneous", |_, this, homogeneous: bool| {
            this.set_column_homogeneous(homogeneous);
            Ok(())
        });

        reg.add_method("set_row_spacing", |_, this, spacing: u32| {
            this.set_row_spacing(spacing);
            Ok(())
        });

        reg.add_method("set_row_homogeneous", |_, this, homogeneous: bool| {
            this.set_row_homogeneous(homogeneous);
            Ok(())
        });

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let grid = Grid::new();
                lua.create_any_userdata(grid)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for CenterBox {
    const CLASS_NAME: &'static str = "CenterBox";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method(
            "set_start_widget",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                this.set_start_widget(child.as_deref());
                Ok(())
            },
        );

        reg.add_method(
            "set_center_widget",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                this.set_center_widget(child.as_deref());
                Ok(())
            },
        );

        reg.add_method(
            "set_end_widget",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                this.set_end_widget(child.as_deref());
                Ok(())
            },
        );

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let center_box = CenterBox::new();
                lua.create_any_userdata(center_box)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for gtk::cairo::Context {
    const CLASS_NAME: &'static str = "CairoContext";
    const CONSTRUCTIBLE: bool = false;

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method(
            "set_source_rgb",
            |_, this, (red, green, blue): (f64, f64, f64)| {
                this.set_source_rgb(red, green, blue);
                Ok(())
            },
        );

        reg.add_method(
            "set_source_rgba",
            |_, this, (red, green, blue, alpha): (f64, f64, f64, f64)| {
                this.set_source_rgba(red, green, blue, alpha);
                Ok(())
            },
        );

        reg.add_method("move_to", |_, this, (x, y): (f64, f64)| {
            this.move_to(x, y);
            Ok(())
        });

        reg.add_method("rel_move_to", |_, this, (dx, dy): (f64, f64)| {
            this.rel_move_to(dx, dy);
            Ok(())
        });

        reg.add_method("line_to", |_, this, (x, y): (f64, f64)| {
            this.line_to(x, y);
            Ok(())
        });

        reg.add_method("rel_line_to", |_, this, (dx, dy): (f64, f64)| {
            this.line_to(dx, dy);
            Ok(())
        });

        reg.add_method(
            "arc",
            |_, this, (xc, yc, radius, angle1, angle2): (f64, f64, f64, f64, f64)| {
                this.arc(xc, yc, radius, angle1, angle2);
                Ok(())
            },
        );

        reg.add_method(
            "arc_negative",
            |_, this, (xc, yc, radius, angle1, angle2): (f64, f64, f64, f64, f64)| {
                this.arc_negative(xc, yc, radius, angle1, angle2);
                Ok(())
            },
        );

        reg.add_method(
            "curve_to",
            |_, this, (x1, y1, x2, y2, x3, y3): (f64, f64, f64, f64, f64, f64)| {
                this.curve_to(x1, y1, x2, y2, x3, y3);
                Ok(())
            },
        );

        reg.add_method(
            "rel_curve_to",
            |_, this, (dx1, dy1, dx2, dy2, dx3, dy3): (f64, f64, f64, f64, f64, f64)| {
                this.rel_curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
                Ok(())
            },
        );

        reg.add_method(
            "rectangle",
            |_, this, (x, y, width, height): (f64, f64, f64, f64)| {
                this.rectangle(x, y, width, height);
                Ok(())
            },
        );

        reg.add_method("translate", |_, this, (tx, ty): (f64, f64)| {
            this.translate(tx, ty);
            Ok(())
        });

        reg.add_method("scale", |_, this, (sx, sy): (f64, f64)| {
            this.scale(sx, sy);
            Ok(())
        });

        reg.add_method("rotate", |_, this, angle: f64| {
            this.rotate(angle);
            Ok(())
        });

        reg.add_method("new_path", |_, this, ()| {
            this.new_path();
            Ok(())
        });

        reg.add_method("new_sub_path", |_, this, ()| {
            this.new_sub_path();
            Ok(())
        });

        reg.add_method("close_path", |_, this, ()| {
            this.close_path();
            Ok(())
        });

        reg.add_method("clip", |_, this, ()| {
            this.clip();
            Ok(())
        });

        reg.add_method("paint", |_, this, ()| this.paint().into_lua_err());
        reg.add_method("paint_with_alpha", |_, this, alpha: f64| {
            this.paint_with_alpha(alpha).into_lua_err()
        });
        reg.add_method("stroke", |_, this, ()| this.stroke().into_lua_err());
        reg.add_method("fill", |_, this, ()| this.fill().into_lua_err());
        reg.add_method("save", |_, this, ()| this.save().into_lua_err());
        reg.add_method("restore", |_, this, ()| this.restore().into_lua_err());
    }
}

impl LuaApi for DrawingArea {
    const CLASS_NAME: &'static str = "DrawingArea";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("set_content_width", |_, this, width: i32| {
            this.set_content_width(width);
            Ok(())
        });

        reg.add_method("set_content_height", |_, this, height: i32| {
            this.set_content_height(height);
            Ok(())
        });

        reg.add_method("set_draw_func", |_, this, f: LuaOwnedFunction| {
            this.set_draw_func(move |_, ctx, width, height| {
                let ctx = ContextWrapper(ctx.clone());
                catch_lua_errors::<_, ()>(f.to_ref(), (ctx, width, height));
            });

            Ok(())
        });

        reg.add_method("unset_draw_func", |_, this, ()| {
            this.unset_draw_func();
            Ok(())
        });

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let drawing_area = DrawingArea::new();
                lua.create_any_userdata(drawing_area)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for Image {
    const CLASS_NAME: &'static str = "Image";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
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

        reg.add_method("set_from_gicon", |_, this, icon: LuaUserDataRef<Icon>| {
            this.set_from_gicon(&*icon);
            Ok(())
        });

        reg.add_method("clear", |_, this, ()| {
            this.clear();
            Ok(())
        });

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let image = Image::new();
                lua.create_any_userdata(image)
            })?,
        )?;
        table.set(
            "from_file",
            lua.create_function(|lua, path: String| {
                let image = Image::from_file(path);
                lua.create_any_userdata(image)
            })?,
        )?;
        table.set(
            "from_icon_name",
            lua.create_function(|lua, icon_name: String| {
                let image = Image::from_icon_name(&icon_name);
                lua.create_any_userdata(image)
            })?,
        )?;
        table.set(
            "from_gicon",
            lua.create_function(|lua, icon: LuaUserDataRef<Icon>| {
                let image = Image::from_gicon(&*icon);
                lua.create_any_userdata(image)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for Scale {
    const CLASS_NAME: &'static str = "Scale";

    fn to_lua_string<'a>(&self, lua: &'a Lua) -> LuaResult<LuaString<'a>> {
        lua.create_string(format!("Scale {{ value = {} }}", self.value()))
    }

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        register_signals!(reg, [value_changed]);

        reg.add_method("connect_adjust_bounds", |_, this, f: LuaOwnedFunction| {
            this.connect_adjust_bounds(move |_, value| {
                catch_lua_errors::<_, ()>(f.to_ref(), value);
            });
            Ok(())
        });

        reg.add_method("set_value", |_, this, value: f64| {
            this.set_value(value);
            Ok(())
        });

        reg.add_method("value", |_, this, ()| Ok(this.value()));

        reg.add_method("set_range", |_, this, (min, max): (f64, f64)| {
            this.set_range(min, max);
            Ok(())
        });

        reg.add_method("set_inverted", |_, this, setting: bool| {
            this.set_inverted(setting);
            Ok(())
        });

        reg.add_method("is_inverted", |_, this, ()| Ok(this.is_inverted()));

        reg.add_method("round_digits", |_, this, ()| Ok(this.round_digits()));

        reg.add_method("set_round_digits", |_, this, round_digits: i32| {
            this.set_round_digits(round_digits);
            Ok(())
        });

        reg.add_method("digits", |_, this, ()| Ok(this.digits()));

        reg.add_method("set_digits", |_, this, digits: i32| {
            this.set_digits(digits);
            Ok(())
        });

        reg.add_method("draws_value", |_, this, ()| Ok(this.draws_value()));

        reg.add_method("set_draw_value", |_, this, draw_value: bool| {
            this.set_draw_value(draw_value);
            Ok(())
        });

        reg.add_method("set_increments", |_, this, (step, page): (f64, f64)| {
            this.set_increments(step, page);
            Ok(())
        });

        reg.add_method("set_fill_level", |_, this, fill_level: f64| {
            this.set_fill_level(fill_level);
            Ok(())
        });

        reg.add_method("fill_level", |_, this, ()| Ok(this.fill_level()));

        reg.add_method(
            "set_restrict_to_fill_level",
            |_, this, restrict_to_fill_level| {
                this.set_restrict_to_fill_level(restrict_to_fill_level);
                Ok(())
            },
        );

        reg.add_method("restricts_to_fill_level", |_, this, ()| {
            Ok(this.restricts_to_fill_level())
        });

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "with_range",
            lua.create_function(
                |lua, (orientation, min, max, step): (enums::Orientation, f64, f64, Option<f64>)| {
                    let scale = Scale::with_range(orientation.0, min, max, step.unwrap_or(1.0));
                    lua.create_any_userdata(scale)
                },
            )?,
        )?;

        Ok(())
    }
}

impl LuaApi for Revealer {
    const CLASS_NAME: &'static str = "Revealer";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method(
            "set_child",
            |_, this, child: Option<LuaUserDataRef<gtk::Widget>>| {
                this.set_child(child.as_deref());
                Ok(())
            },
        );

        reg.add_method("set_reveal_child", |_, this, reveal_child: bool| {
            this.set_reveal_child(reveal_child);
            Ok(())
        });

        reg.add_method("set_transition_duration", |_, this, duration: u32| {
            this.set_transition_duration(duration);
            Ok(())
        });

        reg.add_method(
            "set_transition_type",
            |_, this, transition: enums::RevealerTransitionType| {
                this.set_transition_type(transition.0);
                Ok(())
            },
        );

        add_widget_methods(reg);
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let image = Revealer::new();
                lua.create_any_userdata(image)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for EventControllerKey {
    const CLASS_NAME: &'static str = "EventControllerKey";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("upcast", |lua, this, ()| {
            lua.create_any_userdata(this.clone().upcast::<gtk::EventController>())
        });

        reg.add_method("forward", |_, this, widget: LuaUserDataRef<gtk::Widget>| {
            Ok(this.forward(&*widget))
        });

        reg.add_method("connect_key_pressed", |_, this, f: LuaOwnedFunction| {
            this.connect_key_pressed(move |_, key, keycode, state| {
                let key_name = key.name().map(GStringWrapper);
                let state = ModifierTypeWrapper(state);
                let result = f
                    .call::<_, Option<bool>>((key_name, keycode, state))
                    .unwrap();

                gtk::Inhibit(result.unwrap_or(false))
            });

            Ok(())
        });

        reg.add_method("connect_key_released", |_, this, f: LuaOwnedFunction| {
            this.connect_key_pressed(move |_, key, keycode, state| {
                let key_name = key.name().map(GStringWrapper);
                let state = ModifierTypeWrapper(state);
                let result = f
                    .call::<_, Option<bool>>((key_name, keycode, state))
                    .unwrap();

                gtk::Inhibit(result.unwrap_or(false))
            });

            Ok(())
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let event_controller = EventControllerKey::new();
                lua.create_any_userdata(event_controller)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for EventControllerScroll {
    const CLASS_NAME: &'static str = "EventControllerScroll";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        register_signals!(reg, [scroll_begin, scroll_end]);

        reg.add_method("upcast", |lua, this, ()| {
            lua.create_any_userdata(this.clone().upcast::<gtk::EventController>())
        });

        reg.add_method("connect_scroll", |_, this, f: LuaOwnedFunction| {
            this.connect_scroll(move |_, dx, dy| {
                let result =
                    catch_lua_errors::<_, Option<bool>>(f.to_ref(), (dx, dy)).unwrap_or(None);
                gtk::Inhibit(result.unwrap_or(false))
            });

            Ok(())
        });

        reg.add_method("connect_decelerate", |_, this, f: LuaOwnedFunction| {
            this.connect_decelerate(move |_, vel_x, vel_y| {
                catch_lua_errors::<_, ()>(f.to_ref(), (vel_x, vel_y));
            });

            Ok(())
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, flags: EventControllerScrollFlagsWrapper| {
                let event_controller = EventControllerScroll::new(flags.0);
                lua.create_any_userdata(event_controller)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for EventControllerMotion {
    const CLASS_NAME: &'static str = "EventControllerMotion";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        register_signals!(reg, [leave]);

        reg.add_method("upcast", |lua, this, ()| {
            lua.create_any_userdata(this.clone().upcast::<gtk::EventController>())
        });

        reg.add_method("connect_enter", |_, this, f: LuaOwnedFunction| {
            this.connect_enter(move |_, x, y| {
                catch_lua_errors::<_, ()>(f.to_ref(), (x, y));
            });

            Ok(())
        });

        reg.add_method("connect_motion", |_, this, f: LuaOwnedFunction| {
            this.connect_motion(move |_, x, y| {
                catch_lua_errors::<_, ()>(f.to_ref(), (x, y));
            });

            Ok(())
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let event_controller = EventControllerMotion::new();
                lua.create_any_userdata(event_controller)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for EventControllerFocus {
    const CLASS_NAME: &'static str = "EventControllerFocus";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        register_signals!(reg, [enter, leave]);

        reg.add_method("upcast", |lua, this, ()| {
            lua.create_any_userdata(this.clone().upcast::<gtk::EventController>())
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let event_controller = EventControllerFocus::new();
                lua.create_any_userdata(event_controller)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for Settings {
    const CLASS_NAME: &'static str = "Settings";
    const CONSTRUCTIBLE: bool = false;

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("gtk_cursor_theme_name", |_, this, ()| {
            Ok(this.gtk_cursor_theme_name().map(GStringWrapper))
        });

        reg.add_method(
            "set_gtk_cursor_theme_name",
            |_, this, cursor_theme_name: Option<String>| {
                this.set_gtk_cursor_theme_name(cursor_theme_name.as_deref());
                Ok(())
            },
        );

        reg.add_method("gtk_cursor_theme_size", |_, this, ()| {
            Ok(this.gtk_cursor_theme_size())
        });

        reg.add_method(
            "set_gtk_cursor_theme_size",
            |_, this, cursor_theme_size: i32| {
                this.set_gtk_cursor_theme_size(cursor_theme_size);
                Ok(())
            },
        );

        reg.add_method("gtk_theme_name", |_, this, ()| {
            Ok(this.gtk_theme_name().map(GStringWrapper))
        });

        reg.add_method(
            "set_gtk_theme_name",
            |_, this, theme_name: Option<String>| {
                this.set_gtk_theme_name(theme_name.as_deref());
                Ok(())
            },
        );

        reg.add_method("gtk_icon_theme_name", |_, this, ()| {
            Ok(this.gtk_icon_theme_name().map(GStringWrapper))
        });

        reg.add_method(
            "set_gtk_icon_theme_name",
            |_, this, icon_theme_name: Option<String>| {
                this.set_gtk_icon_theme_name(icon_theme_name.as_deref());
                Ok(())
            },
        );
    }
}

impl LuaApi for CssProvider {
    const CLASS_NAME: &'static str = "CssProvider";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("load_from_data", |_, this, data: String| {
            this.load_from_data(&data);
            Ok(())
        });

        reg.add_method("load_from_path", |_, this, path: String| {
            this.load_from_path(path);
            Ok(())
        });
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let provider = CssProvider::new();
                lua.create_any_userdata(provider)
            })?,
        )?;

        Ok(())
    }
}

impl LuaApi for MainContext {
    const CLASS_NAME: &'static str = "MainContext";

    fn register_methods(reg: &mut LuaUserDataRegistry<Self>) {
        reg.add_method("spawn_local", |_, this, f: LuaOwnedFunction| {
            this.spawn_local(async move {
                catch_lua_errors_async::<_, ()>(f.to_ref(), ()).await;
            });
            Ok(())
        });

        reg.add_method(
            "spawn_local_with_priority",
            |_, this, (priority, f): (LuaUserDataRef<glib::Priority>, LuaOwnedFunction)| {
                this.spawn_local_with_priority(*priority, async move {
                    catch_lua_errors_async::<_, ()>(f.to_ref(), ()).await;
                });
                Ok(())
            },
        );
    }

    fn register_static_methods(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
        table.set(
            "new",
            lua.create_function(|lua, ()| {
                let ctx = MainContext::new();
                lua.create_any_userdata(ctx)
            })?,
        )?;
        table.set(
            "default",
            lua.create_function(|lua, ()| {
                let ctx = MainContext::default();
                lua.create_any_userdata(ctx)
            })?,
        )?;
        table.set(
            "thread_default",
            lua.create_function(|lua, ()| {
                Ok(if let Some(ctx) = MainContext::thread_default() {
                    Some(lua.create_any_userdata(ctx)?)
                } else {
                    None
                })
            })?,
        )?;

        Ok(())
    }
}

fn push_layer_shell_api(lua: &Lua, gtk_table: &LuaTable) -> LuaResult<()> {
    let layer_shell = lua.create_table()?;

    push_enum!(
        lua,
        layer_shell,
        gtk4_layer_shell,
        Layer,
        [Background, Bottom, Top, Overlay]
    );

    push_enum!(
        lua,
        layer_shell,
        gtk4_layer_shell,
        Edge,
        [Left, Right, Top, Bottom]
    );

    push_enum!(
        lua,
        layer_shell,
        gtk4_layer_shell,
        KeyboardMode,
        [None, Exclusive, OnDemand]
    );

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
    layer_shell.set(
        "set_keyboard_mode",
        lua.create_function(
            |_, (window, mode): (LuaUserDataRef<ApplicationWindow>, enums::KeyboardMode)| {
                gtk4_layer_shell::set_keyboard_mode(&*window, mode.0);
                Ok(())
            },
        )?,
    )?;
    layer_shell.set(
        "set_namespace",
        lua.create_function(
            |_, (window, namespace): (LuaUserDataRef<ApplicationWindow>, String)| {
                gtk4_layer_shell::set_namespace(&*window, &namespace);
                Ok(())
            },
        )?,
    )?;

    gtk_table.set("layer_shell", layer_shell)?;

    Ok(())
}

pub fn push_api(lua: &Lua, table: &LuaTable) -> LuaResult<()> {
    let gtk_table = lua.create_table()?;

    push_enums(lua, &gtk_table)?;
    push_constants(lua, &gtk_table)?;
    add_global_functions(lua, &gtk_table)?;
    Application::push_lua(lua, &gtk_table)?;
    ApplicationWindow::push_lua(lua, &gtk_table)?;
    Overlay::push_lua(lua, &gtk_table)?;
    Label::push_lua(lua, &gtk_table)?;
    Entry::push_lua(lua, &gtk_table)?;
    Button::push_lua(lua, &gtk_table)?;
    CheckButton::push_lua(lua, &gtk_table)?;
    Box::push_lua(lua, &gtk_table)?;
    Grid::push_lua(lua, &gtk_table)?;
    CenterBox::push_lua(lua, &gtk_table)?;
    DrawingArea::push_lua(lua, &gtk_table)?;
    Image::push_lua(lua, &gtk_table)?;
    Scale::push_lua(lua, &gtk_table)?;
    Revealer::push_lua(lua, &gtk_table)?;
    EventControllerKey::push_lua(lua, &gtk_table)?;
    EventControllerScroll::push_lua(lua, &gtk_table)?;
    EventControllerMotion::push_lua(lua, &gtk_table)?;
    EventControllerFocus::push_lua(lua, &gtk_table)?;
    Settings::push_lua(lua, &gtk_table)?;
    CssProvider::push_lua(lua, &gtk_table)?;
    MainContext::push_lua(lua, &gtk_table)?;
    push_layer_shell_api(lua, &gtk_table)?;

    table.set("gtk", gtk_table)?;

    Ok(())
}
