---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local

local gdk = {}

---@class RGBA
---@field r number?
---@field g number?
---@field b number?
---@field a number?

---@class Texture
gdk.Texture = {
    ---@param data string
    ---@return Texture
    from_bytes = function(data) end,

    ---@param path string
    ---@return Texture
    from_filename = function(path) end,
}

crabshell.gdk = gdk
