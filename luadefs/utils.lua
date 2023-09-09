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

    ---@param tbl table
    ---@param seen? table
    ---@param depth? integer
    print_table = function(tbl, seen, depth) end
}

---@class RegexMatch
---@field start_index integer
---@field end_index integer
---@field is_empty boolean
---@field str string

---@alias RegexCaptures table<number | string, RegexMatch?>

---@class Regex
utils.Regex = {
    -- Syntax: https://docs.rs/regex/latest/regex/#syntax
    ---@param re string
    ---@return Regex
    new = function(re) end,

    ---@param self Regex
    ---@param haystack string
    ---@param start_index? integer
    ---@return boolean
    is_match = function(self, haystack, start_index) end,

    ---@param self Regex
    ---@param haystack string
    ---@param start_index? integer
    ---@return RegexMatch?
    find = function(self, haystack, start_index) end,

    ---@param self Regex
    ---@param haystack string
    ---@param start_index? integer
    ---@return RegexCaptures?
    captures = function(self, haystack, start_index) end,

    -- String interpolation syntax:
    -- https://docs.rs/regex/latest/regex/struct.Regex.html#replacement-string-syntax
    ---@param self Regex
    ---@param haystack string
    ---@param rep string|fun(caps: RegexCaptures):string
    ---@return string
    replace_all = function(self, haystack, rep) end,
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
