use serde::{Deserialize, Serialize};

pub trait Command {
    const NAME: &'static str;
}

#[derive(Serialize, Deserialize)]
pub struct Workspace {
    pub id: u64,
    pub name: String,
    pub monitor: String,
    pub windows: u64,
    pub hasfullscreen: bool,
    pub lastwindow: String,
    pub lastwindowtitle: String,
}

pub type Workspaces = Vec<Workspace>;
impl Command for Vec<Workspace> {
    const NAME: &'static str = "workspaces";
}

#[derive(Serialize, Deserialize)]
pub struct Mouse {
    pub address: String,
    pub name: String,
    pub defaultSpeed: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Keyboard {
    pub address: String,
    pub name: String,
    pub rules: String,
    pub model: String,
    pub layout: String,
    pub variant: String,
    pub options: String,
    pub active_keymap: String,
    pub main: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TabletOwner {
    pub address: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tablet {
    pub address: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub belongsTo: TabletOwner,
}

#[derive(Serialize, Deserialize)]
pub struct Touch {
    pub address: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Switch {
    pub address: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Devices {
    pub mice: Vec<Mouse>,
    pub keyboards: Vec<Keyboard>,
    pub tablets: Vec<Tablet>,
    pub touch: Vec<Touch>,
    pub switches: Vec<Switch>,
}

impl Command for Devices {
    const NAME: &'static str = "devices";
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialWorkspace {
    pub address: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ActiveWindow {
    pub address: Option<String>,
    pub mapped: Option<bool>,
    pub hidden: Option<bool>,
    pub at: Option<(u64, u64)>,
    pub size: Option<(u64, u64)>,
    pub workspace: Option<PartialWorkspace>,
    pub floating: Option<bool>,
    pub monitor: Option<u64>,
    pub class: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "initialClass")]
    pub initial_class: Option<String>,
    #[serde(rename = "initialTitle")]
    pub initial_title: Option<String>,
    pub pid: Option<u64>,
    pub xwayland: Option<bool>,
    pub pinned: Option<bool>,
    pub fullscreen: Option<bool>,
    #[serde(rename = "fullscreenMode")]
    pub fullscreen_mode: Option<i32>,
    #[serde(rename = "fakeFullscreen")]
    pub fake_fullscreen: Option<bool>,
    pub grouped: Option<Vec<String>>,
    pub swallowing: Option<String>,
}

impl Command for ActiveWindow {
    const NAME: &'static str = "activewindow";
}
