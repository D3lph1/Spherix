use std::net::SocketAddr;

pub struct Connection {
    pub id: i32,
    addr: SocketAddr,
}

impl Connection {
    pub fn new(id: i32, addr: SocketAddr) -> Self {
        Self {
            id,
            addr
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
