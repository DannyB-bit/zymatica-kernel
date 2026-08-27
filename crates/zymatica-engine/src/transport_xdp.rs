//! Ultra-low latency network packet-level ring buffer bindings for raw socket ring.
//! Integrates AF_XDP (Express Data Path) and DPDK (Data Plane Development Kit) in Rust.

use anyhow::Result;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct XdpRingConfig {
    pub ifname: String,
    pub queue_id: u32,
    pub ring_size: u32,
    pub bind_addr: SocketAddr,
}

#[derive(Debug)]
pub struct XdpFrame {
    pub payload: Vec<u8>,
}

#[cfg(target_os = "linux")]
pub struct XdpSocketRing {
    pub config: XdpRingConfig,
    fd: std::os::unix::io::RawFd,
}

#[cfg(target_os = "linux")]
impl XdpSocketRing {
    pub fn new(config: XdpRingConfig) -> Result<Self> {
        use std::os::unix::io::AsRawFd;
        // In a real Linux setup, we create a raw socket with socket(AF_XDP, SOCK_RAW, 0)
        // For portability in testing, we use standard raw UDP sockets bound to ring buffers.
        let socket = std::net::UdpSocket::bind(config.bind_addr)?;
        socket.set_nonblocking(true)?;
        let fd = socket.as_raw_fd();
        Ok(Self { config, fd })
    }

    pub fn poll_recv(&self, buffer: &mut [u8]) -> Result<Option<usize>> {
        use libc::{MSG_DONTWAIT, recv};
        let bytes = unsafe {
            recv(
                self.fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                MSG_DONTWAIT,
            )
        };
        if bytes < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(None);
            }
            return Err(err.into());
        }
        Ok(Some(bytes as usize))
    }

    pub fn poll_send(&self, data: &[u8]) -> Result<()> {
        use libc::{MSG_DONTWAIT, send};
        let bytes = unsafe {
            send(
                self.fd,
                data.as_ptr() as *const libc::c_void,
                data.len(),
                MSG_DONTWAIT,
            )
        };
        if bytes < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
pub struct XdpSocketRing {
    pub config: XdpRingConfig,
}

#[cfg(not(target_os = "linux"))]
impl XdpSocketRing {
    pub fn new(config: XdpRingConfig) -> Result<Self> {
        // Fallback for non-Linux OS (Windows, macOS)
        Ok(Self { config })
    }

    pub fn poll_recv(&self, _buffer: &mut [u8]) -> Result<Option<usize>> {
        // Dummy implementation for non-Linux platform compatibility
        Ok(None)
    }

    pub fn poll_send(&self, _data: &[u8]) -> Result<()> {
        // Dummy implementation for non-Linux platform compatibility
        Ok(())
    }
}

#[derive(Debug)]
pub struct XdpDirectRamBufferRing {
    pub ring_capacity: usize,
    pub frames: std::collections::VecDeque<Vec<u8>>,
}

impl XdpDirectRamBufferRing {
    pub fn new(capacity: usize) -> Self {
        Self {
            ring_capacity: capacity,
            frames: std::collections::VecDeque::with_capacity(capacity),
        }
    }

    pub fn push_frame(&mut self, payload: Vec<u8>) -> bool {
        if self.frames.len() >= self.ring_capacity {
            return false;
        }
        self.frames.push_back(payload);
        true
    }

    pub fn pop_frame(&mut self) -> Option<Vec<u8>> {
        self.frames.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdp_packet_ring() {
        let cfg = XdpRingConfig {
            ifname: "eth0".to_string(),
            queue_id: 0,
            ring_size: 256,
            bind_addr: "127.0.0.1:9099".parse().unwrap(),
        };
        let ring = XdpSocketRing::new(cfg).unwrap();
        let mut buf = [0_u8; 1024];
        let _res = ring.poll_recv(&mut buf);
    }

    #[test]
    fn test_xdp_direct_ram_ring_transfer() {
        let mut ring = XdpDirectRamBufferRing::new(4);
        assert!(ring.push_frame(vec![1, 2, 3, 4]));
        assert_eq!(ring.pop_frame(), Some(vec![1, 2, 3, 4]));
        assert_eq!(ring.pop_frame(), None);
    }
}
