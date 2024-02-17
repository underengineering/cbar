use futures::io::AsyncBufReadExt;
use gtk::{
    gio::{
        IOStreamAsyncReadWrite, InputStreamAsyncBufRead, PollableInputStream, SocketClient,
        SocketConnection, UnixSocketAddress,
    },
    prelude::*,
};
use std::{env, path::Path};
use tokio::sync::broadcast;

use super::error::Error;
use super::events::{Event, ScreenCastOwner};

pub struct EventLoop {
    stream: Option<IOStreamAsyncReadWrite<SocketConnection>>,
    reader: Option<InputStreamAsyncBufRead<PollableInputStream>>,
    sender: broadcast::Sender<Event>,
    _receiver: broadcast::Receiver<Event>,
}

impl EventLoop {
    /// Returns a new unconnected event loop
    pub fn new() -> Self {
        let (sender, receiver) = broadcast::channel(24);
        Self {
            stream: None,
            reader: None,
            sender,
            _receiver: receiver,
        }
    }

    /// Connects the event loop
    pub async fn connect(&mut self) -> Result<(), Error> {
        let hyprctl_instance_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("Failed to get the hyprland instance signature");

        let socket2_path = format!("/tmp/hypr/{hyprctl_instance_sig}/.socket2.sock");
        let socket2_path = Path::new(&socket2_path);

        let sock = SocketClient::new();
        let conn = sock
            .connect_future(&UnixSocketAddress::new(socket2_path))
            .await?;
        let stream = conn.into_async_read_write().unwrap();

        self.reader = Some(stream.input_stream().clone().into_async_buf_read(256));
        self.stream = Some(stream);

        Ok(())
    }

    pub fn connected(&self) -> bool {
        self.reader.is_some()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    async fn next(&mut self) -> Result<Event, Error> {
        let reader = self
            .reader
            .as_mut()
            .expect("Event loop must be connected before usage");

        let mut line = String::with_capacity(64);
        reader.read_line(&mut line).await?;
        let line = line.trim_end();

        let (event_name, event_data) = line.split_once(">>").unwrap();
        Ok(match event_name {
            "workspace" => Event::Workspace {
                name: event_data.into(),
            },
            "focusedmon" => {
                let (monitor, workspace) = event_data.split_once(',').unwrap();
                Event::FocusedMonitor {
                    monitor: monitor.into(),
                    workspace: workspace.into(),
                }
            }
            "activewindow" => {
                let (class, title) = event_data.split_once(',').unwrap();
                Event::ActiveWindow {
                    class: class.into(),
                    title: title.into(),
                }
            }
            "activewindowv2" => Event::ActiveWindowV2 {
                address: if event_data == "," {
                    None
                } else {
                    Some(usize::from_str_radix(event_data, 16).unwrap())
                },
            },
            "fullscreen" => Event::FullScreen {
                active: event_data == "1",
            },
            "monitorremoved" => Event::MonitorRemoved {
                monitor: event_data.into(),
            },
            "monitoradded" => Event::MonitorAdded {
                monitor: event_data.into(),
            },
            "createworkspace" => Event::CreateWorkspace {
                name: event_data.into(),
            },
            "destroyworkspace" => Event::DestroyWorkspace {
                name: event_data.into(),
            },
            "moveworkspace" => {
                let (workspace, monitor) = event_data.split_once(',').unwrap();
                Event::MoveWorkspace {
                    workspace: workspace.into(),
                    monitor: monitor.into(),
                }
            }
            "activelayout" => {
                let (keyboard_name, layout_name) = event_data.split_once(',').unwrap();
                Event::ActiveLayout {
                    keyboard_name: keyboard_name.into(),
                    layout_name: layout_name.into(),
                }
            }
            "openwindow" => {
                let mut iter = event_data.splitn(4, ',');
                let address = iter.next().unwrap();
                let workspace = iter.next().unwrap();
                let class = iter.next().unwrap();
                let title = iter.next().unwrap();
                Event::OpenWindow {
                    address: usize::from_str_radix(address, 16).unwrap(),
                    workspace: workspace.into(),
                    class: class.into(),
                    title: title.into(),
                }
            }
            "closewindow" => Event::CloseWindow {
                address: usize::from_str_radix(event_data, 16).unwrap(),
            },
            "movewindow" => {
                let (address, workspace) = event_data.split_once(',').unwrap();
                Event::MoveWindow {
                    address: usize::from_str_radix(address, 16).unwrap(),
                    workspace: workspace.into(),
                }
            }
            "openlayer" => Event::OpenLayer {
                name: event_data.into(),
            },
            "closelayer" => Event::CloseLayer {
                name: event_data.into(),
            },
            "submap" => Event::SubMap {
                name: event_data.into(),
            },
            "changefloatingmode" => {
                let (address, active) = event_data.split_once(',').unwrap();
                Event::ChangeFloatingMode {
                    address: usize::from_str_radix(address, 16).unwrap(),
                    active: active == "1",
                }
            }
            "urgent" => Event::Urgent {
                address: usize::from_str_radix(event_data, 16).unwrap(),
            },
            "minimize" => {
                let (address, active) = event_data.split_once(',').unwrap();
                Event::Minimize {
                    address: usize::from_str_radix(address, 16).unwrap(),
                    active: active == "1",
                }
            }
            "screencast" => {
                let (state, owner) = event_data.split_once(',').unwrap();
                Event::ScreenCast {
                    state: state == "1",
                    owner: if owner == "0" {
                        ScreenCastOwner::Monitor
                    } else {
                        ScreenCastOwner::Window
                    },
                }
            }
            "windowtitle" => Event::WindowTitle {
                address: usize::from_str_radix(event_data, 16).unwrap(),
            },
            "ignoregrouplock" => Event::IgnoreGroupLock {
                ignore: event_data == "1",
            },
            "lockgroups" => Event::LockGroups {
                lock: event_data == "1",
            },
            "configreloaded" => Event::ConfigReloaded,
            "monitoraddedv2" => {
                let mut iter = event_data.splitn(3, ',');
                let id = iter.next().unwrap();
                let name = iter.next().unwrap();
                let description = iter.next().unwrap();
                Event::MonitorAddedV2 {
                    id: id.parse::<u64>().unwrap(),
                    name: name.to_string(),
                    description: description.to_string(),
                }
            }
            _ => Event::Unknown {
                raw: line.to_string(),
            },
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            let event = self.next().await?;
            self.sender.send(event)?;
        }
    }
}
