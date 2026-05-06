//! Low-level TCP tuning helpers.

use socket2::SockRef;
use std::io;

/// Disable Nagle's algorithm for latency-sensitive streams.
pub fn set_tcp_nodelay(stream: &std::net::TcpStream) -> io::Result<()> {
    let sock = SockRef::from(stream);
    sock.set_nodelay(true)
}
