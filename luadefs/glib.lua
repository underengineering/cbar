---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local
local glib     = {}

---@class Priority
local Priority = {}

glib.Priority  = {
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
