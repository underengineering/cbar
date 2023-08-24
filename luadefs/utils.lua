---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local

---@class IconLookupOptions
---@field size integer?
---@field scale integer?
---@field theme string?
---@field cache boolean?
---@field force_svg boolean?

utils = {
    ---@param path string
    ---@return string
    scss_from_path = function(path) end,

    ---@param input string
    ---@return string
    scss_from_string = function(input) end,

    ---@param name string
    ---@param options IconLookupOptions
    ---@return string?
    lookup_icon = function(name, options) end,

    ---@async
    ---@param secs number
    sleep = function(secs) end,
}

utils.json = {
    ---@param tbl table
    ---@return string
    to_string = function(tbl) end,

    ---@param tbl table
    ---@return string
    to_string_pretty = function(tbl) end,

    ---@param str string
    ---@return table
    from_str = function(str) end,
}
