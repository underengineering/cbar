use serde::{Deserialize, Serialize};
use std::{cell::RefCell, env, rc::Rc};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        unix::{OwnedReadHalf, OwnedWriteHalf},
        UnixStream,
    },
};

use super::error::Error;

pub mod commands;
use self::commands::Command;

async fn try_request<T: Command>(buffer: &mut Vec<u8>) -> Result<(), Error> {
    let hyprctl_instance_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .expect("Failed to get the hyprland instance signature");

    let socket_path = format!("/tmp/hypr/{hyprctl_instance_sig}/.socket.sock");
    let mut stream = UnixStream::connect(&socket_path).await?;

    stream
        .write_all(format!("j/{}", T::NAME).as_bytes())
        .await?;
    stream.flush().await?;

    stream.read_to_end(buffer).await?;

    Ok(())
}

pub async fn request<'a, T: Deserialize<'a> + Command>(
    buffer: &'a mut Vec<u8>,
) -> Result<T, Error> {
    // 3 retries
    for _ in 0..3 {
        match try_request::<T>(buffer).await {
            Ok(_) => return Ok(serde_json::from_slice(buffer)?),
            Err(Error::Io(err)) if err.kind() == io::ErrorKind::BrokenPipe => continue, // Retry
            Err(err) => return Err(err),
        }
    }

    Err(Error::MaxRetriesExceeded())
}
