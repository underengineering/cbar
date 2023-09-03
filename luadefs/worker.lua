---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local

worker = {}

---@alias WorkerData nil | boolean | integer | number | string | WorkerData[]

---@class WorkerSender
local WorkerSender = {
    ---@param self WorkerSender
    ---@param value WorkerData
    send = function(self, value) end,

    -- Tries to send a value to the worker.
    -- Returns whether the value has been sent
    ---@param self WorkerSender
    ---@param value WorkerData
    ---@return boolean
    try_send = function(self, value) end
}

---@class WorkerReceiver
local WorkerReceiver = {
    -- Receives data from workers channel.
    -- Propagates any error that was generated.
    -- Returns nil if worker has stopped
    ---@param self WorkerReceiver
    ---@return WorkerData?
    recv = function(self) end,

    -- Receives data from workers channel.
    -- Propagates any error that was generated.
    -- Returns false,nil if no data is available
    ---@param self WorkerReceiver
    ---@return boolean,WorkerData?
    try_recv = function(self) end
}

---@class Worker
worker.Worker = {
    ---@param code string Code to be passed to the created lua vm
    ---@param name? string Compiled chunk name
    ---@return Worker
    start = function(code, name) end,

    -- Returns whether worker has terminated
    ---@param self Worker
    ---@return boolean
    dead = function(self) end,

    -- Waits for worker termination. Propagates any error that was generated
    ---@param self Worker
    join = function(self) end,

    ---@param self Worker
    ---@return WorkerSender
    sender = function(self) end,

    ---@param self Worker
    ---@return WorkerReceiver
    receiver = function(self) end,
}
