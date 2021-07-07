use std::net::{SocketAddr, TcpStream};
use std::io::{Write, Read, Error};

#[derive(Copy, Clone)]
pub struct TcpHandle(usize);

pub trait TcpSocket {
    /// Write a slice to the given handle
    fn write_all(&mut self, data: &[u8]);

    /// Attempt to read from the socket into buf, will return the about of data received or None
    fn try_read(&mut self, buf: &mut [u8]) -> Option<usize>;
}

pub trait TcpBackend {
    /// Create a connection to a given address
    /// Will either return a handle to a socket, or None if the connection failed
    fn connect(&mut self, addr: SocketAddr) -> Option<Box<dyn TcpSocket>>;
}

// pub struct NullTcpBackend;
//
// impl TcpBackend for NullTcpBackend {
//     fn connect(&mut self, addr: SocketAddr) -> Option<TcpHandle> {
//         None
//     }
// }
//
#[derive(Default)]
pub struct DesktopTcpBackend;

impl TcpBackend for DesktopTcpBackend {
    fn connect(&mut self, addr: SocketAddr) -> Option<Box<dyn TcpSocket>> {
        if let Ok(tcp) = TcpStream::connect(addr) {
            tcp.set_nonblocking(true);
            return Some(Box::new(DesktopTcpSocket {
                socket: tcp
            }));
        } else {
            return None;
        }
    }
}

struct DesktopTcpSocket {
    socket: TcpStream,
}

impl TcpSocket for DesktopTcpSocket {
    fn write_all(&mut self, data: &[u8]) {
        self.socket.write_all(data);
    }

    fn try_read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        let mut buf = [0u8; 4096];
        match self.socket.read(&mut buf) {
            Ok(len) => {
                let data = &buf[..len];
                (buffer[..len]).copy_from_slice(data);
                Some(len)
            }
            Err(_) => {
                None
            }
        }
    }
}
