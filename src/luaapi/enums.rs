use mlua::{prelude::*, IntoLua};
use thiserror::Error;

macro_rules! impl_lua {
    ($typ:ty) => {
        impl<'lua> IntoLua<'lua> for $typ {
            fn into_lua(self, _lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
                let i: i32 = self.into();
                Ok(LuaValue::Integer(i as i64))
            }
        }

        impl<'lua> FromLua<'lua> for $typ {
            fn from_lua(value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
                if let LuaValue::Integer(value) = value {
                    Ok(<$typ>::try_from(value as i32).unwrap())
                } else {
                    panic!(
                        "Invalid type for enum '{}', expected number, got: {:?}",
                        stringify!($typ),
                        value
                    );
                }
            }
        }
    };
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Conversion failed")]
    ConversionFailed,
}

pub(super) struct Orientation(pub(super) gtk::Orientation);
impl std::convert::TryFrom<i32> for Orientation {
    type Error = Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Orientation(gtk::Orientation::Horizontal)),
            1 => Ok(Orientation(gtk::Orientation::Vertical)),
            _ => Err(Error::ConversionFailed),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<i32> for Orientation {
    fn into(self) -> i32 {
        match self.0 {
            gtk::Orientation::Horizontal => 0,
            gtk::Orientation::Vertical => 1,
            _ => unreachable!(),
        }
    }
}

impl_lua!(Orientation);

pub(super) struct Layer(pub(super) gtk4_layer_shell::Layer);
impl std::convert::TryFrom<i32> for Layer {
    type Error = Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Layer(gtk4_layer_shell::Layer::Background)),
            1 => Ok(Layer(gtk4_layer_shell::Layer::Bottom)),
            2 => Ok(Layer(gtk4_layer_shell::Layer::Top)),
            3 => Ok(Layer(gtk4_layer_shell::Layer::Overlay)),
            _ => Err(Error::ConversionFailed),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<i32> for Layer {
    fn into(self) -> i32 {
        match self.0 {
            gtk4_layer_shell::Layer::Background => 0,
            gtk4_layer_shell::Layer::Bottom => 1,
            gtk4_layer_shell::Layer::Top => 2,
            gtk4_layer_shell::Layer::Overlay => 3,
            _ => unreachable!(),
        }
    }
}

impl_lua!(Layer);

pub(super) struct Edge(pub(super) gtk4_layer_shell::Edge);
impl std::convert::TryFrom<i32> for Edge {
    type Error = Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Edge(gtk4_layer_shell::Edge::Left)),
            1 => Ok(Edge(gtk4_layer_shell::Edge::Right)),
            2 => Ok(Edge(gtk4_layer_shell::Edge::Top)),
            3 => Ok(Edge(gtk4_layer_shell::Edge::Bottom)),
            _ => Err(Error::ConversionFailed),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<i32> for Edge {
    fn into(self) -> i32 {
        match self.0 {
            gtk4_layer_shell::Edge::Left => 0,
            gtk4_layer_shell::Edge::Right => 1,
            gtk4_layer_shell::Edge::Top => 2,
            gtk4_layer_shell::Edge::Bottom => 3,
            _ => unreachable!(),
        }
    }
}

impl_lua!(Edge);
