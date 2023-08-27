use gtk::{gdk::ModifierType, glib::GString};
use mlua::prelude::*;
use paste::paste;

use crate::utils::unpack_mask_postfixed;

pub struct GStringWrapper(pub GString);
impl<'lua> IntoLua<'lua> for GStringWrapper {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue> {
        Ok(LuaValue::String(lua.create_string(self.0.as_str())?))
    }
}

pub struct ModifierTypeWrapper(pub ModifierType);
impl<'lua> IntoLua<'lua> for ModifierTypeWrapper {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        unpack_mask_postfixed!(
            table,
            self.0,
            ModifierType,
            [
                SHIFT, LOCK, CONTROL, ALT, BUTTON1, BUTTON2, BUTTON3, BUTTON4, BUTTON5, SUPER,
                HYPER, META
            ],
            _MASK
        );
        Ok(LuaValue::Table(table))
    }
}
