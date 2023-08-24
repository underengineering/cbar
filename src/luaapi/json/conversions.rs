use mlua::prelude::*;
use serde_json::{Map, Number, Value};

use super::error::Error;

pub trait TryJsonFrom<T> {
    type Error;
    fn try_json_from(value: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl<'lua> TryJsonFrom<LuaValue<'lua>> for Value {
    type Error = Error;

    fn try_json_from(value: LuaValue<'lua>) -> Result<Self, Self::Error> {
        match value {
            LuaValue::String(str) => Ok(Value::String(String::from(str.to_str()?))),
            LuaValue::Nil => Ok(Value::Null),
            LuaValue::Boolean(value) => Ok(Value::Bool(value)),
            LuaValue::LightUserData(_) => Err(Error::JsonConversion(format!("{:?}", value))),
            LuaValue::Integer(value) => Ok(Value::Number(Number::from(value))),
            LuaValue::Number(value) => {
                if let Some(value) = Number::from_f64(value) {
                    Ok(Value::Number(value))
                } else {
                    Err(Error::JsonInvalidNumber(value))
                }
            }
            LuaValue::Table(table) => Value::try_json_from(table),
            LuaValue::Function(_) => Err(Error::JsonConversion(format!("{:?}", value))),
            LuaValue::Error(_) => Err(Error::JsonConversion(format!("{:?}", value))),
            LuaValue::Thread(_) => Err(Error::JsonConversion(format!("{:?}", value))),
            LuaValue::UserData(_) => Err(Error::JsonConversion(format!("{:?}", value))),
        }
    }
}

impl<'lua> TryJsonFrom<LuaTable<'lua>> for Value {
    type Error = Error;

    fn try_json_from(table: LuaTable<'lua>) -> Result<Self, Self::Error> {
        let array_len = table.raw_len();
        let total_len = table.clone().pairs::<LuaValue, LuaValue>().count();
        let is_seq = array_len == total_len;

        let value = if is_seq {
            Value::Array(try_lua_table_to_array(table)?)
        } else {
            Value::Object(try_lua_table_to_object(table)?)
        };

        Ok(value)
    }
}

pub trait TryFromJson<'lua> {
    type Error;
    fn try_from_json(lua: &'lua Lua, value: &Value) -> Result<LuaValue<'lua>, Self::Error>;
}

impl<'lua> TryFromJson<'lua> for LuaValue<'lua> {
    type Error = Error;

    fn try_from_json(
        lua: &'lua Lua,
        value: &Value,
    ) -> Result<LuaValue<'lua>, <Self as TryFromJson<'lua>>::Error> {
        match value {
            &Value::Null => Ok(LuaValue::Nil),
            &Value::Bool(value) => Ok(LuaValue::Boolean(value)),
            Value::Number(value) => Ok(if let Some(float) = value.as_f64() {
                LuaValue::Number(float)
            } else {
                let int = value
                    .as_i64()
                    .unwrap_or_else(|| value.as_u64().unwrap() as i64);
                LuaValue::Integer(int)
            }),
            Value::Array(vec) => {
                let table = lua.create_table_with_capacity(vec.len(), 0)?;
                for (idx, value) in vec.iter().enumerate() {
                    table.set(idx + 1, LuaValue::try_from_json(lua, value)?)?;
                }

                Ok(LuaValue::Table(table))
            }
            Value::Object(map) => {
                let table = lua.create_table_with_capacity(0, map.len())?;
                for (key, value) in map.iter() {
                    table.set(key.clone(), LuaValue::try_from_json(lua, value)?)?;
                }

                Ok(LuaValue::Table(table))
            }
            Value::String(str) => Ok(LuaValue::String(lua.create_string(str)?)),
        }
    }
}

fn try_lua_table_to_array(table: LuaTable) -> Result<Vec<Value>, Error> {
    let mut arr = Vec::with_capacity(table.raw_len());
    for value in table.sequence_values::<LuaValue>() {
        arr.push(Value::try_json_from(value?)?)
    }

    Ok(arr)
}

fn try_lua_table_to_object(table: LuaTable) -> Result<Map<String, Value>, Error> {
    let mut obj = Map::new();
    for kv in table.pairs::<LuaString, LuaValue>() {
        let (key, value) = kv?;
        obj.insert(
            String::from(key.to_str().unwrap()),
            Value::try_json_from(value)?,
        );
    }

    Ok(obj)
}
