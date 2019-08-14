use crate::socket::Socket;

pub struct UdpSocket<SocketImpl: Socket> {
    socket: SocketImpl,
}

impl<SocketImpl: Socket> UdpSocket<SocketImpl> {
	pub fn new(socket: SocketImpl) -> Self {
		// TODO initialize socket for UDP mode
		UdpSocket { socket }
	}
}
