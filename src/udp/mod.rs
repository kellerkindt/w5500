use crate::socket::Socket;

pub struct UdpSocket<SocketImpl: Socket> {
    pub socket: SocketImpl,
}
