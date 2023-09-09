---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local

local worker = {}

---@type WorkerSenderSlave?
worker.sender = nil

---@type WorkerSenderSlave?
worker.receiver = nil

---@alias WorkerData nil | boolean | integer | number | string | WorkerData[]

---@class WorkerSenderMaster
local WorkerSenderMaster = {
    ---@param self WorkerSenderMaster
    ---@param value WorkerData
    send = function(self, value) end,

    -- Tries to send a value to the worker.
    -- Returns whether the value has been sent
    ---@param self WorkerSenderMaster
    ---@param value WorkerData
    ---@return boolean
    try_send = function(self, value) end
}

---@class WorkerReceiverMaster
local WorkerReceiverMaster = {
    -- Receives data from worker's channel.
    -- Propagates any error that was generated.
    -- Returns nil if worker has stopped
    ---@param self WorkerReceiverMaster
    ---@return WorkerData?
    recv = function(self) end,

    -- Receives data from worker's channel.
    -- Propagates any error that was generated.
    -- Returns false,nil if no data is available
    ---@param self WorkerReceiverMaster
    ---@return boolean
    ---@return WorkerData?
    try_recv = function(self) end
}

---@class WorkerSenderSlave
local WorkerSenderSlave = {
    ---@param self WorkerSenderSlave
    ---@param value WorkerData
    send = function(self, value) end,

    -- Tries to send a value to the worker's owner.
    -- Returns whether the value has been sent
    ---@param self WorkerSenderSlave
    ---@param value WorkerData
    ---@return boolean
    try_send = function(self, value) end
}

---@class WorkerReceiverSlave
local WorkerReceiverSlave = {
    -- Receives data from worker's owner channel.
    -- Propagates any error that was generated.
    -- Returns nil if worker has stopped
    ---@param self WorkerReceiverSlave
    ---@return WorkerData?
    recv = function(self) end,

    -- Receives data from workers channel.
    -- Returns false,nil if no data is available
    ---@param self WorkerReceiverSlave
    ---@return boolean
    ---@return WorkerData?
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

    -- Waits for worker termination, returning immediately if it's dead.
    -- Returns all collected data in a table. Propagates any error that was generated.
    ---@param self Worker
    ---@return WorkerData[]?
    join = function(self) end,

    ---@param self Worker
    ---@return WorkerSenderMaster
    sender = function(self) end,

    ---@param self Worker
    ---@return WorkerReceiverMaster
    receiver = function(self) end,
}

crabshell.worker = worker
