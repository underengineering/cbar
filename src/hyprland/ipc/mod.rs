use futures::{
    io::{self},
    AsyncReadExt, AsyncWriteExt,
};
use gtk::{
    gio::{SocketClient, UnixSocketAddress},
    prelude::*,
};
use serde::Deserialize;
use std::{env, path::Path};

use super::error::Error;

pub mod commands;
use self::commands::Command;

async fn try_request<T: Command>(buffer: &mut Vec<u8>) -> Result<(), Error> {
    let hyprctl_instance_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .expect("Failed to get the hyprland instance signature");

    let socket_path = format!("/tmp/hypr/{hyprctl_instance_sig}/.socket.sock");
    let socket_path = Path::new(&socket_path);

    let sock = SocketClient::new();
    let conn = sock
        .connect_future(&UnixSocketAddress::new(socket_path))
        .await?;
    let stream = conn.into_async_read_write().unwrap();

    let mut writer = stream.output_stream().clone().into_async_write().unwrap();
    let mut reader = stream.input_stream().clone().into_async_read().unwrap();

    writer
        .write_all(format!("j/{}", T::NAME).as_bytes())
        .await?;

    reader.read_to_end(buffer).await?;

    Ok(())
}

pub async fn request<'a, T: Deserialize<'a> + Command>(
    buffer: &'a mut Vec<u8>,
) -> Result<T, Error> {
    // 6 retries
    for _ in 0..6 {
        match try_request::<T>(buffer).await {
            Ok(_) => return Ok(serde_json::from_slice(buffer)?),
            Err(Error::Io(err)) if err.kind() == io::ErrorKind::BrokenPipe => continue, // Retry
            Err(err) => return Err(err),
        }
    }

    Err(Error::MaxRetriesExceeded())
}
