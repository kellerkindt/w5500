use crate::socket::Socket;

pub struct UdpSocket<'a, SocketImpl: Socket> {
    socket: &'a mut SocketImpl,
}

impl<'a, SocketImpl: Socket> UdpSocket<'a, SocketImpl> {
    pub fn new(socket: &'a mut SocketImpl) -> Self {
        // TODO initialize socket for UDP mode
        UdpSocket { socket }
    }
}
