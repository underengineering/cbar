---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local
local utf8 = {}

---@param str string
---@return integer
function utf8.len(str) end

---@param str string
---@param i integer
---@param j? integer
---@return string
function utf8.sub(str, i, j) end

crabshell.utf8 = utf8
