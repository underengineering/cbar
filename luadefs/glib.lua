---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local
local glib       = {}

---@class Priority
local Priority   = {}

glib.Priority    = {
    ---@type Priority
    HIGH = nil,
    ---@type Priority
    DEFAULT = nil,
    ---@type Priority
    HIGH_IDLE = nil,
    ---@type Priority
    DEFAULT_IDLE = nil,
    ---@type Priority
    LOW = nil,
}

---@class Value
glib.Value       = {
    ---@param str string
    ---@return Value
    new_str = function(str) end,

    ---@param table string[]
    ---@return Value
    new_str_vec = function(table) end,

    ---@param bytes string
    ---@return Value
    new_bytes = function(bytes) end,

    ---@param bool boolean
    ---@return Value
    new_bool = function(bool) end,

    ---@param value number
    ---@return Value
    new_f64 = function(value) end,

    ---@param value number
    ---@return Value
    new_f32 = function(value) end,

    ---@param value integer
    ---@return Value
    new_i64 = function(value) end,

    ---@param value integer
    ---@return Value
    new_i32 = function(value) end,

    ---@param value integer
    ---@return Value
    new_i8 = function(value) end,

    ---@param value integer
    ---@return Value
    new_u64 = function(value) end,

    ---@param value integer
    ---@return Value
    new_u32 = function(value) end,

    ---@param value integer
    ---@return Value
    new_u8 = function(value) end,

    ---@param self Value
    ---@return string
    type_name = function(self) end,

    ---@param self Value
    ---@return string?
    as_str = function(self) end,

    ---@param self Value
    ---@return string[]?
    as_str_vec = function(self) end,

    ---@param self Value
    ---@return string?
    as_bytes = function(self) end,

    ---@param self Value
    ---@return boolean?
    as_bool = function(self) end,

    ---@param self Value
    ---@return number?
    as_f64 = function(self) end,

    ---@param self Value
    ---@return number?
    as_f32 = function(self) end,

    ---@param self Value
    ---@return number?
    as_i64 = function(self) end,

    ---@param self Value
    ---@return number?
    as_i32 = function(self) end,

    ---@param self Value
    ---@return number?
    as_i8 = function(self) end,

    ---@param self Value
    ---@return number?
    as_u64 = function(self) end,

    ---@param self Value
    ---@return number?
    as_u32 = function(self) end,

    ---@param self Value
    ---@return number?
    as_u8 = function(self) end,

}

---@class MainContext
glib.MainContext = {
    ---@return MainContext
    new = function() end,

    ---@return MainContext
    default = function() end,

    ---@return MainContext?
    thread_default = function() end,

    ---@param self MainContext
    ---@param callback fun():nil
    spawn_local = function(self, callback) end,

    ---@param self MainContext
    ---@param priority Priority
    ---@param callback fun():nil
    spawn_local_with_priority = function(self, priority, callback) end
}

crabshell.glib   = glib
