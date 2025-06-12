use std::net::{TcpStream, TcpListener};

pub struct FSRTCPListener {
    inner_listener: TcpListener
}

pub struct FSRTCPConnector {
    inner_stream: TcpStream
}