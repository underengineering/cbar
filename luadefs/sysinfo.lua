---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local
local sysinfo = {}

---@class CpuRefreshKind
---@field frequency boolean?
---@field cpu_usage boolean?

---@class ProcessRefreshKind
---@field cpu boolean?
---@field disk_usage boolean?
---@field user boolean?

---@class RefreshKind
---@field networks boolean?
---@field networks_list boolean?
---@field disks boolean?
---@field disks_list boolean?
---@field memory boolean?
---@field components boolean?
---@field components_list boolean?
---@field users_list boolean?
---@field cpu CpuRefreshKind?
---@field processes ProcessRefreshKind?

---@class Cpu
---@field name string
---@field frequency integer
---@field vendor_id string
---@field brand string
---@field cpu_usage number

---@class NetworkData
---@field received integer
---@field total_received integer
---@field transmitted integer
---@field total_transmitted integer
---@field packets_received integer
---@field total_packets_received integer
---@field packets_transmitted integer
---@field total_packets_transmitted integer
---@field errors_on_received integer
---@field total_errors_on_received integer
---@field errors_on_transmitted integer
---@field total_errors_on_transmitted integer
---@field mac_address table<integer, integer>

---@class System
sysinfo.System = {
    ---@return System
    new_all = function() end,

    ---@param refreshes? RefreshKind
    ---@return System
    new_with_specifics = function(refreshes) end,

    ---@param self System
    refresh_all = function(self) end,

    ---@param self System
    refresh_system = function(self) end,

    ---@param self System
    refresh_memory = function(self) end,

    ---@param self System
    refresh_cpu = function(self) end,

    ---@param self System
    ---@param kind? CpuRefreshKind
    refresh_cpu_specifics = function(self, kind) end,

    ---@param self System
    ---@param pid number
    refresh_process = function(self, pid) end,

    ---@param self System
    ---@param kind? ProcessRefreshKind
    refresh_processes_specifics = function(self, kind) end,

    ---@param self System
    ---@param pid number
    ---@param kind? ProcessRefreshKind
    refresh_process_specifics = function(self, pid, kind) end,

    ---@param self System
    refresh_disks = function(self) end,

    ---@param self System
    refresh_disks_list = function(self) end,

    ---@param self System
    refresh_users_list = function(self) end,

    ---@param self System
    refresh_networks = function(self) end,

    ---@param self System
    refresh_networks_list = function(self) end,

    ---@param self System
    refresh_components = function(self) end,

    ---@param self System
    refresh_components_list = function(self) end,

    ---@param self System
    ---@return Cpu[]
    cpus = function(self) end,

    ---@param self System
    ---@return Cpu
    global_cpu_info = function(self) end,

    ---@param self System
    ---@return integer
    total_memory = function(self) end,

    ---@param self System
    ---@return integer
    used_memory = function(self) end,

    ---@param self System
    ---@return integer
    free_memory = function(self) end,

    ---@param self System
    ---@return integer
    total_swap = function(self) end,

    ---@param self System
    ---@return integer
    free_swap = function(self) end,

    ---@param self System
    ---@return integer
    used_swap = function(self) end,

    ---@param self System
    ---@return table<string, NetworkData>
    networks = function(self) end,
}

---@class Duration
---@field secs integer
---@field nanos integer

---@class BatteryInfo
---@field capacity integer
---@field full number
---@field now number
---@field current number
---@field remaining_time Duration
---@field status string

---@class Batteries
---@field info table<string, BatteryInfo>
---@field total_capacity integer
---@field remaining_time Duration

sysinfo.battery = {
    ---@return boolean
    is_on_ac = function() end,

    ---@return Batteries
    get_batteries = function() end
}

crabshell.sysinfo = sysinfo
