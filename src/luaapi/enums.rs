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
                    Ok(
                        <$typ>::try_from(value as i32)
                            .expect("Failed to convert enum from integer"),
                    )
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
            0 => Ok(Self(gtk::Orientation::Horizontal)),
            1 => Ok(Self(gtk::Orientation::Vertical)),
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
            0 => Ok(Self(gtk4_layer_shell::Layer::Background)),
            1 => Ok(Self(gtk4_layer_shell::Layer::Bottom)),
            2 => Ok(Self(gtk4_layer_shell::Layer::Top)),
            3 => Ok(Self(gtk4_layer_shell::Layer::Overlay)),
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
            0 => Ok(Self(gtk4_layer_shell::Edge::Left)),
            1 => Ok(Self(gtk4_layer_shell::Edge::Right)),
            2 => Ok(Self(gtk4_layer_shell::Edge::Top)),
            3 => Ok(Self(gtk4_layer_shell::Edge::Bottom)),
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

pub(super) struct KeyboardMode(pub(super) gtk4_layer_shell::KeyboardMode);
impl std::convert::TryFrom<i32> for KeyboardMode {
    type Error = Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self(gtk4_layer_shell::KeyboardMode::None)),
            1 => Ok(Self(gtk4_layer_shell::KeyboardMode::Exclusive)),
            2 => Ok(Self(gtk4_layer_shell::KeyboardMode::OnDemand)),
            _ => Err(Error::ConversionFailed),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<i32> for KeyboardMode {
    fn into(self) -> i32 {
        match self.0 {
            gtk4_layer_shell::KeyboardMode::None => 0,
            gtk4_layer_shell::KeyboardMode::Exclusive => 1,
            gtk4_layer_shell::KeyboardMode::OnDemand => 2,
            _ => unreachable!(),
        }
    }
}

impl_lua!(KeyboardMode);

pub(super) struct Align(pub(super) gtk::Align);
impl std::convert::TryFrom<i32> for Align {
    type Error = Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self(gtk::Align::Fill)),
            1 => Ok(Self(gtk::Align::Start)),
            2 => Ok(Self(gtk::Align::End)),
            3 => Ok(Self(gtk::Align::Center)),
            4 => Ok(Self(gtk::Align::Baseline)),
            _ => Err(Error::ConversionFailed),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<i32> for Align {
    fn into(self) -> i32 {
        match self.0 {
            gtk::Align::Fill => 0,
            gtk::Align::Start => 1,
            gtk::Align::End => 2,
            gtk::Align::Center => 3,
            gtk::Align::Baseline => 4,
            _ => unreachable!(),
        }
    }
}

impl_lua!(Align);

pub(super) struct RevealerTransitionType(pub(super) gtk::RevealerTransitionType);
impl std::convert::TryFrom<i32> for RevealerTransitionType {
    type Error = Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self(gtk::RevealerTransitionType::None)),
            1 => Ok(Self(gtk::RevealerTransitionType::Crossfade)),
            2 => Ok(Self(gtk::RevealerTransitionType::SlideRight)),
            3 => Ok(Self(gtk::RevealerTransitionType::SlideLeft)),
            4 => Ok(Self(gtk::RevealerTransitionType::SlideUp)),
            5 => Ok(Self(gtk::RevealerTransitionType::SlideDown)),
            6 => Ok(Self(gtk::RevealerTransitionType::SwingRight)),
            7 => Ok(Self(gtk::RevealerTransitionType::SwingLeft)),
            8 => Ok(Self(gtk::RevealerTransitionType::SwingUp)),
            9 => Ok(Self(gtk::RevealerTransitionType::SwingDown)),
            _ => Err(Error::ConversionFailed),
        }
    }
}
#[allow(clippy::from_over_into)]
impl Into<i32> for RevealerTransitionType {
    fn into(self) -> i32 {
        match self.0 {
            gtk::RevealerTransitionType::None => 0,
            gtk::RevealerTransitionType::Crossfade => 1,
            gtk::RevealerTransitionType::SlideRight => 2,
            gtk::RevealerTransitionType::SlideLeft => 3,
            gtk::RevealerTransitionType::SlideUp => 4,
            gtk::RevealerTransitionType::SlideDown => 5,
            gtk::RevealerTransitionType::SwingRight => 6,
            gtk::RevealerTransitionType::SwingLeft => 7,
            gtk::RevealerTransitionType::SwingUp => 8,
            gtk::RevealerTransitionType::SwingDown => 9,
            _ => unreachable!(),
        }
    }
}

impl_lua!(RevealerTransitionType);
