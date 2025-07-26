use adb_client::{ADBDeviceExt, ADBServer, ADBServerDevice};
use serde::Serialize;
use tokio::{
    sync::mpsc::{self, UnboundedSender},
    task::JoinHandle,
};

use std::io::{Result as IoResult, Write};
use std::{
    fs::File,
    io::Cursor,
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
};

#[derive(Clone, Debug, Serialize)]
pub struct Device {
    pub id: String,
    pub status: String,
}

impl Device {
    fn new_server_device(id: &str) -> ADBServerDevice {
        ADBServerDevice::new(id.to_string(), None)
    }

    pub fn push(id: &str, src: &str, des: &str) -> Result<(), String> {
        let mut device = Device::new_server_device(id);
        let mut input = File::open(Path::new(src))
            .map_err(|e| format!("Failed to open file '{}': {}", src, e))?;
        device
            .push(&mut input, des)
            .map_err(|e| format!("Failed to push file '{}' to '{}': {}", src, des, e))?;
        Ok(())
    }

    pub fn pull(id: &str, src: String, output: &mut dyn Write) -> Result<(), String> {
        let mut device = Device::new_server_device(id);
        device
            .pull(&src, output)
            .map_err(|e| format!("Failed to pull file '{}' from device '{}': {}", src, id, e))
    }

    pub fn reverse(id: &str, remote: &str, local: &str) -> Result<(), String> {
        let mut device = Device::new_server_device(id);
        device
            .reverse(remote.to_string(), local.to_string())
            .map_err(|e| format!("Failed to reverse '{}' to '{}': {}", remote, local, e))
    }

    pub fn forward(id: &str, local: &str, remote: &str) -> Result<(), String> {
        let mut device = Device::new_server_device(id);
        device
            .forward(remote.to_string(), local.to_string())
            .map_err(|e| format!("Failed to forward '{}' to '{}': {}", local, remote, e))
    }

    pub fn shell_process<S>(id: &str, shell_args: S) -> JoinHandle<Result<(), String>>
    where
        S: IntoIterator,
        S::Item: Into<String>,
    {
        let mut device = Device::new_server_device(id);
        let shell_args: Vec<String> = shell_args.into_iter().map(|s| s.into()).collect();

        let (tx, mut rx) = mpsc::unbounded_channel();
        let h: JoinHandle<Result<(), String>> = tokio::task::spawn_blocking(move || {
            let mut writer = ChannelWriter { sender: tx };
            let shell_args: Vec<&str> = shell_args.iter().map(|s| s.as_str()).collect();
            device.shell_command(&shell_args, &mut writer).map_err(|e| {
                let msg = format!("Failed to run adb shell command: {}", e);
                log::error!("[Adb] {}", msg);
                msg
            })
        });

        tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                log::info!("[Adb] {}", line);
            }
        });

        h
    }

    pub fn shell<S>(id: &str, shell_args: S, output: &mut dyn Write) -> Result<(), String>
    where
        S: IntoIterator,
        S::Item: Into<String>,
    {
        let mut device = Device::new_server_device(id);
        let shell_args: Vec<String> = shell_args.into_iter().map(|s| s.into()).collect();
        let shell_args: Vec<&str> = shell_args.iter().map(|s| s.as_str()).collect();
        device
            .shell_command(&shell_args, output)
            .map_err(|e| e.to_string())
    }

    pub fn screen_size(id: &str) -> Result<(u32, u32), String> {
        let mut device = Device::new_server_device(id);

        let mut output: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut output);
        device
            .shell_command(&["wm", "size"], &mut cursor)
            .map_err(|e| format!("Failed to run adb shell command: {}", e))?;

        let stdout = String::from_utf8_lossy(&output);
        for line in stdout.lines() {
            if let Some(rest) = line.strip_prefix("Physical size: ") {
                let mut parts = rest.trim().split('x');
                let width = parts
                    .next()
                    .ok_or("Missing width")?
                    .parse::<u32>()
                    .map_err(|e| format!("Failed to parse width: {}", e))?;

                let height = parts
                    .next()
                    .ok_or("Missing height")?
                    .parse::<u32>()
                    .map_err(|e| format!("Failed to parse height: {}", e))?;
                return Ok((width, height));
            }
        }
        Err("Failed to get screen size".to_string())
    }
}

pub struct Adb {
    pub server: ADBServer,
    pub adb_path: String,
}

impl Adb {
    pub fn check_adb_path(adb_path: &str) -> Result<(), String> {
        match which::which(&adb_path) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("adb not found: {}", adb_path)),
        }
    }

    pub fn new(adb_path: String) -> Adb {
        Self {
            server: ADBServer::new_from_path(
                SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037),
                Some(adb_path.clone()),
            ),
            adb_path,
        }
    }

    pub fn devices(&mut self) -> Result<Vec<Device>, String> {
        let device_list = self.server.devices().map_err(|e| e.to_string())?;
        Ok(device_list
            .iter()
            .map(|d| Device {
                id: d.identifier.clone(),
                status: d.state.to_string(),
            })
            .collect::<Vec<_>>())
    }

    pub fn kill_server(&mut self) -> Result<(), String> {
        self.server
            .kill()
            .map_err(|e| format!("Failed to kill adb server': {}", e))
    }

    pub fn connect_device(&mut self, address: &str) -> Result<(), String> {
        let socket_addr = address
            .parse::<SocketAddrV4>()
            .map_err(|e| format!("Failed to parse device address: {}", e))?;

        self.server
            .connect_device(socket_addr)
            .map_err(|e| format!("Failed to connect to device '{}': {}", address, e))
    }

    pub fn pair_device(&mut self, address: &str, code: &str) -> Result<(), String> {
        let socket_addr = address
            .parse::<SocketAddrV4>()
            .map_err(|e| format!("Failed to parse device address: {}", e))?;

        self.server
            .pair(socket_addr, code.to_string())
            .map_err(|e| format!("Failed to pair with device '{}': {}", address, e))
    }
}

struct ChannelWriter {
    pub sender: UnboundedSender<String>,
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            for line in s.lines() {
                let _ = self.sender.send(line.to_string());
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}
