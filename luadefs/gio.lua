---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local
local gio = {}

---@class InputStreamAsyncBufRead
local InputStreamAsyncBufRead = {
    ---@param self InputStreamAsyncBufRead
    ---@param capacity? integer
    ---@return string
    read_line = function(self, capacity) end,

    ---@param self InputStreamAsyncBufRead
    ---@param size integer
    ---@return string
    read_exact = function(self, size) end,

    ---@param self InputStreamAsyncBufRead
    ---@param size integer
    ---@return string
    read = function(self, size) end,

    ---@param self InputStreamAsyncBufRead
    ---@param size integer
    ---@return string
    read_to_end = function(self, size) end,
}

---@class InputStream
local InputStream = {
    ---@param self InputStream
    ---@param buffer_size integer
    ---@return InputStreamAsyncBufRead
    into_async_buf_read = function(self, buffer_size) end,

    ---@async
    ---@param self InputStream
    ---@param count integer
    ---@return string
    read = function(self, count) end,

    ---@async
    ---@param self InputStream
    ---@param count integer
    skip = function(self, count) end,

    ---@async
    ---@param self InputStream
    close = function(self) end
}

---@class OutputStream
local OutputStream = {
    ---@async
    ---@param self OutputStream
    ---@param data string
    ---@return integer
    write = function(self, data) end,

    ---@async
    ---@param self OutputStream
    flush = function(self) end,

    ---@param self OutputStream
    ---@return boolean
    has_pending = function(self) end,

    ---@param self OutputStream
    ---@return boolean
    is_closed = function(self) end,

    ---@param self OutputStream
    ---@return boolean
    is_closing = function(self) end,

    ---@async
    ---@param self OutputStream
    close = function(self) end,
}

---@class SubprocessFlags
---@field stdin_pipe boolean?
---@field stdin_inherit boolean?
---@field stdout_pipe boolean?
---@field stdout_silence boolean?
---@field stderr_pipe boolean?
---@field stderr_silence boolean?
---@field stderr_merge boolean?
---@field inherit_fds boolean?

---@class Subprocess
gio.Subprocess = {
    ---@param args string[]
    ---@param flags? SubprocessFlags
    ---@return Subprocess
    new = function(args, flags) end,

    ---@async
    ---@param self Subprocess
    ---@param data? string
    ---@return string?
    ---@return string?
    communicate_raw = function(self, data) end,

    ---@async
    ---@param self Subprocess
    ---@param data string?
    ---@return string?
    ---@return string?
    communicate = function(self, data) end,

    ---@async
    ---@param self Subprocess
    wait = function(self) end,

    ---@param self Subprocess
    ---@param signal_num integer
    send_signal = function(self, signal_num) end,

    ---@param self Subprocess
    ---@return string
    identifier = function(self) end,

    ---@param self Subprocess
    force_kill = function(self) end,

    ---@param self Subprocess
    ---@return boolean
    has_exited = function(self) end,

    ---@param self Subprocess
    ---@return integer
    exit_status = function(self) end,

    ---@param self Subprocess
    ---@return OutputStream?
    stdin = function(self) end,

    ---@param self Subprocess
    ---@return InputStream?
    stdout = function(self) end,

    ---@param self Subprocess
    ---@return InputStream?
    stderr = function(self) end,
}


---@class SocketConnection
local SocketConnection = {
    ---@param self SocketConnection
    ---@return InputStream
    input_stream = function(self) end,

    ---@param self SocketConnection
    ---@return OutputStream
    output_stream = function(self) end,

    ---@async
    ---@param self SocketConnection
    close = function(self) end
}

---@class SocketClient
gio.SocketClient = {
    ---@return SocketClient
    new = function() end,

    ---@async
    ---@param self SocketClient
    ---@param path string
    ---@return SocketConnection
    connect_unix = function(self, path) end
}

---@class File
gio.File = {
    ---@param path string
    ---@return File
    for_path = function(path) end,

    ---@param self File
    ---@return string?
    path = function(self) end,

    ---@async
    ---@param self File
    ---@return InputStream
    read = function(self) end,

    ---@async
    ---@param self File
    ---@return OutputStream
    create = function(self) end,

    ---@async
    ---@param self File
    ---@return OutputStream
    replace = function(self) end,
}

---@class ThemedIcon
gio.ThemedIcon = {
    ---@param self ThemedIcon
    ---@return string[]
    names = function(self) end
}

---@class Icon
gio.Icon = {
    ---@param self Icon
    ---@return ThemedIcon
    as_themed = function(self) end
}

---@class AppInfoMonitor
gio.AppInfoMonitor = {
    ---@return AppInfoMonitor
    get = function() end,

    ---@param self AppInfoMonitor
    ---@param callback fun():nil
    connect_changed = function(self, callback) end,
}

---@class Icon
local Icon = {}

---@class AppInfo
gio.AppInfo = {
    ---@return AppInfo[]
    all = function() end,

    ---@param self AppInfo
    ---@return string
    name = function(self) end,

    ---@param self AppInfo
    ---@return string
    display_name = function(self) end,

    ---@param self AppInfo
    ---@return Icon?
    icon = function(self) end,

    ---@param self AppInfo
    ---@return string?
    id = function(self) end,

    ---@param self AppInfo
    ---@return string?
    description = function(self) end,

    ---@param self AppInfo
    delete = function(self) end,

    ---@param self AppInfo
    ---@param files File[]
    launch = function(self, files) end,

    ---@param self AppInfo
    ---@param uris string[]
    launch_uris = function(self, uris) end,

    ---@param self AppInfo
    ---@return boolean
    should_show = function(self) end,

    ---@param self AppInfo
    ---@return boolean
    supports_files = function(self) end,

    ---@param self AppInfo
    ---@return boolean
    supports_uris = function(self) end,

    ---@param self AppInfo
    ---@return boolean
    can_delete = function(self) end,
}

crabshell.gio = gio
