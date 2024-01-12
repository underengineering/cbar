use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScreenCastOwner {
    Monitor,
    Window,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    Workspace {
        name: String,
    },
    FocusedMonitor {
        monitor: String,
        workspace: String,
    },
    ActiveWindow {
        class: String,
        title: String,
    },
    ActiveWindowV2 {
        address: Option<usize>,
    },
    FullScreen {
        active: bool,
    },
    MonitorRemoved {
        monitor: String,
    },
    MonitorAdded {
        monitor: String,
    },
    CreateWorkspace {
        name: String,
    },
    DestroyWorkspace {
        name: String,
    },
    MoveWorkspace {
        workspace: String,
        monitor: String,
    },
    ActiveLayout {
        keyboard_name: String,
        layout_name: String,
    },
    OpenWindow {
        address: usize,
        workspace: String,
        class: String,
        title: String,
    },
    CloseWindow {
        address: usize,
    },
    MoveWindow {
        address: usize,
        workspace: String,
    },
    OpenLayer {
        name: String,
    },
    CloseLayer {
        name: String,
    },
    SubMap {
        name: String,
    },
    ChangeFloatingMode {
        address: usize,
        active: bool,
    },
    Urgent {
        address: usize,
    },
    Minimize {
        address: usize,
        active: bool,
    },
    ScreenCast {
        state: bool,
        owner: ScreenCastOwner,
    },
    WindowTitle {
        address: usize,
    },
    IgnoreGroupLock {
        ignore: bool,
    },
    LockGroups {
        lock: bool,
    },
    ConfigReloaded,
}
