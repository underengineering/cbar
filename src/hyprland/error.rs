use gtk::glib;
use thiserror::Error;
use tokio::{io, sync::broadcast};

use super::events::Event;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] io::Error),
    #[error("GLib error")]
    GLib(#[from] glib::Error),
    #[error("Channel error")]
    ChannelError(#[from] broadcast::error::SendError<Event>),
    #[error("JSON parsing error")]
    JsonParse(#[from] serde_json::Error),
    #[error("Max retries exceeded")]
    MaxRetriesExceeded(),
}
