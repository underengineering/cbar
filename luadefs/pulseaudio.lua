---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local

---@module 'gtk.lua'

pulseaudio = {}

---@enum State
pulseaudio.State = {
    Unconnected = 0,
    Connecting = 1,
    Authorizing = 2,
    SettingName = 3,
    Ready = 4,
    Failed = 5,
    Terminated = 6
}

---@enum Facility
pulseaudio.Facility = {
    Sink = 0,
    Source = 1,
    SinkInput = 2,
    SourceOutput = 3,
    Module = 4,
    Client = 5,
    SampleCache = 6,
    Server = 7,
    Card = 9
}

---@enum Operation
pulseaudio.Operation = {
    New = 0,
    Changed = 16,
    Removed = 32
}


---@class Mainloop
pulseaudio.Mainloop = {
    ---@param ctx MainContext?
    ---@return Mainloop
    new = function(ctx) end
}

---@class SinkInfo
---@field name string
---@field index integer
---@field volume integer[]
---@field mute boolean
---@field base_volume integer

---@class ServerInfo
---@field user_name string
---@field host_name string
---@field server_version string
---@field server_name string
---@field default_sink_name string
---@field default_source_name string

---@class InterestMaskSetMasks
---@field sink boolean?
---@field source boolean?
---@field sink_input boolean?
---@field source_output boolean?
---@field module boolean?
---@field client boolean?
---@field sample_cache boolean?
---@field server boolean?
---@field card boolean?
---@field all boolean?

---@class InterestMaskSet
pulseaudio.InterestMaskSet = {
    ---@param masks InterestMaskSetMasks
    ---@return InterestMaskSet
    new = function(masks) end
}

---@class Volume
---@field NORMAL integer
---@field MUTED integer
---@field MAX integer
---@field INTEGER integer
pulseaudio.Volume = {}

---@class Context
pulseaudio.Context = {
    ---@param mainloop Mainloop
    ---@param name string
    ---@return Context
    new = function(mainloop, name) end,

    ---@param self Context
    ---@param server boolean?
    connect = function(self, server) end,

    ---@param self Context
    ---@param callback fun()
    set_state_callback = function(self, callback) end,

    ---@param self Context
    ---@return State
    get_state = function(self) end,

    ---@param self Context
    ---@param mask InterestMaskSet
    ---@param callback fun(success: boolean)
    subscribe = function(self, mask, callback) end,

    ---@param self Context
    ---@param callback fun(facility: Facility, op: Operation, index: integer)
    set_subscribe_callback = function(self, callback) end,

    ---@param self Context
    ---@param callback fun(info: ServerInfo)
    get_server_info = function(self, callback) end,

    ---@param self Context
    ---@param index integer
    ---@param callback fun(sink: SinkInfo)
    get_sink_info_by_index = function(self, index, callback) end,

    ---@param self Context
    ---@param name string
    ---@param callback fun(sink: SinkInfo)
    get_sink_info_by_name = function(self, name, callback) end,
}
