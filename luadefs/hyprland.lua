---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local
hyprland = {}

---@alias CommandName `workspaces` | `devices` | `activewindow`

---@class Workspace
---@field id number
---@field name string
---@field monitor string
---@field windows number
---@field hasfullscreen boolean
---@field lastwindow string
---@field lastwindowtitle string

---@alias Workspaces Workspace[]

---@class Mouse
---@field address string
---@field name string
---@field default_speed number

---@class Keyboard
---@field address string
---@field name string
---@field rules string
---@field model string
---@field layout string
---@field variant string
---@field options string
---@field active_keymap string
---@field main boolean

---@class TabletOwner
---@field address string
---@field name string

---@class Tablet
---@field address string
---@field type_ string
---@field belongs_to TabletOwner

---@class Touch
---@field address string
---@field name string

---@class Switch
---@field address string
---@field name string

---@class Devices
---@field mice Mouse[]
---@field keyboards Keyboard[]
---@field tablets Tablet[]
---@field touch Touch[]
---@field switches Switch[]

---@class PartialWorkspace
---@field address string
---@field name string

---@class ActiveWindow
---@field address string?
---@field mapped boolean?
---@field hidden boolean?
---@field at { [0]: integer, [1]: integer }?
---@field size { [0]: integer, [1]: integer }?
---@field workspace PartialWorkspace?
---@field floating boolean?
---@field monitor number?
---@field class string?
---@field title string?
---@field initialClass string?
---@field initialTitle string?
---@field pid integer?
---@field xwayland boolean?
---@field pinned boolean?
---@field fullscreen boolean?
---@field fullscreen_mode integer?
---@field fakeFullscreen boolean?
---@field grouped string[]?
---@field swallowing string?

---@class Monitor
---@field id integer
---@field name string
---@field description string
---@field make string
---@field model string
---@field serial string
---@field width integer
---@field height integer
---@field refreshRate number
---@field x integer
---@field y integer
---@field activeWorkspace PartialWorkspace
---@field specialWorkspace PartialWorkspace
---@field reserved { [1]: integer, [2]: integer, [3]: integer, [4]: integer }
---@field scale number
---@field transform integer
---@field focused boolean
---@field dpmsStatus boolean
---@field vrr boolean

---@alias Monitors Monitor[]

hyprland.ipc = {}

---@async
---@return Workspaces
function hyprland.ipc.get_workspaces() end

---@async
---@return Devices
function hyprland.ipc.get_devices() end

---@async
---@return ActiveWindow
function hyprland.ipc.get_activewindow() end

---@async
---@return Monitors
function hyprland.ipc.get_monitors() end

---@enum ScreenCastOwner
local ScreenCastOwner = {
    Monitor = 0,
    Window = 1,
}

---@class Event
---@field Workspace { name: string }
---@field FocusedMonitor { monitor: string, workspace: string }
---@field ActiveWindow { class: string, title: string }
---@field ActiveWindowV2 { address: number|nil }
---@field FullScreen { active: boolean }
---@field MonitorRemoved { monitor: string }
---@field MonitorAdded { monitor: string }
---@field CreateWorkspace { name: string }
---@field DestroyWorkspace { name: string }
---@field MoveWorkspace { workspace: string, monitor: string }
---@field ActiveLayout { keyboard_name: string, layout_name: string }
---@field OpenWindow { address: number, workspace: string, class: string, title: string }
---@field CloseWindow { address: number }
---@field MoveWindow { address: number, workspace: string }
---@field OpenLayer { name: string }
---@field CloseLayer { name: string }
---@field SubMap { name: string }
---@field ChangeFloatingMode { address: number, active: boolean }
---@field Urgent { address: number }
---@field Minimize { address: number, active: boolean }
---@field ScreenCast { state: boolean, owner: ScreenCastOwner }
---@field WindowTitle { address: number }

---@class Receiver
local Receiver = {
    ---@async
    ---@return Event?
    recv = function() end
}

---@class EventLoop
hyprland.EventLoop = {
    ---@return EventLoop
    new = function() end,

    ---@async
    ---@param self EventLoop
    connect = function(self) end,

    ---@param self EventLoop
    ---@return Receiver
    subscribe = function(self) end,

    ---@async
    ---@param self EventLoop
    run = function(self) end
}
