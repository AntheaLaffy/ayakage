use crate::config::Action;
use anyhow::{Context, Result};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::Duration,
};

pub const PORT: u16 = 11574;
pub struct Control {
    socket: UdpSocket,
}
impl Control {
    pub fn new(port: u16) -> Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, port)).with_context(|| {
            format!("无法打开本机快捷键接口 127.0.0.1:{port}；可能已有 Ayakage 在运行")
        })?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket })
    }
    pub fn port(&self) -> u16 {
        self.socket
            .local_addr()
            .expect("bound control socket")
            .port()
    }
    pub fn poll(&self) -> Vec<(Action, SocketAddr)> {
        let mut buffer = [0u8; 4096];
        let mut actions = vec![];
        for _ in 0..32 {
            match self.socket.recv_from(&mut buffer) {
                Ok((length, sender)) => {
                    if let Ok(command) = std::str::from_utf8(&buffer[..length]) {
                        match Action::from_command(command) {
                            Ok(action) => actions.push((action, sender)),
                            Err(_) => {
                                let _ = self.socket.send_to(b"error: unknown command", sender);
                            }
                        }
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => {
                    log::warn!("control receiver: {error}");
                    break;
                }
            }
        }
        actions
    }
    pub fn reply(&self, recipient: SocketAddr, success: bool) {
        let _ = self.socket.send_to(
            if success {
                b"ok"
            } else {
                b"error: action failed; see studio"
            },
            recipient,
        );
    }
}
pub fn send(command: &str, port: u16) -> Result<()> {
    let action = Action::from_command(command)?;
    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))?;
    socket.connect((Ipv4Addr::LOCALHOST, port))?;
    socket.set_read_timeout(Some(Duration::from_secs(2)))?;
    socket.send(action.command().as_bytes())?;
    let mut response = [0u8; 256];
    let length = socket
        .recv(&mut response)
        .context("Ayakage 没有响应，请先启动桌面应用")?;
    anyhow::ensure!(
        &response[..length] == b"ok",
        "命令执行失败：{}",
        String::from_utf8_lossy(&response[..length])
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn isolated_command_transport_reports_the_execution_result() {
        for success in [true, false] {
            let receiver = Control::new(0).unwrap();
            let port = receiver.port();
            let worker = std::thread::spawn(move || {
                let deadline = std::time::Instant::now() + Duration::from_secs(2);
                loop {
                    if let Some((action, sender)) = receiver.poll().into_iter().next() {
                        assert_eq!(action, Action::ResetView);
                        receiver.reply(sender, success);
                        return;
                    }
                    assert!(std::time::Instant::now() < deadline, "no command received");
                    std::thread::sleep(Duration::from_millis(2));
                }
            });
            assert_eq!(send("reset-view", port).is_ok(), success);
            worker.join().unwrap();
        }
    }
}
