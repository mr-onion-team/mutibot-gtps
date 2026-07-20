use std::io::{Read, Write};
use std::net::TcpStream;
use tungstenite::protocol::Message;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

pub struct WsBotHost {
    ws: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    addr: std::net::SocketAddr,
    host_header: String,
    pending_connect: bool,
    connected: bool,
}

#[derive(Debug)]
pub enum WsPacketEvent {
    Connect,
    Receive(Vec<u8>),
    Disconnect,
    None,
}

impl WsBotHost {
    pub fn new(addr: std::net::SocketAddr, host_header: String) -> Self {
        Self {
            ws: None,
            addr,
            host_header,
            pending_connect: true,
            connected: false,
        }
    }

    pub fn connect_now(&mut self) -> bool {
        let url = format!("ws://{}", self.addr);
        let host = self.host_header.clone();

        let request = tungstenite::client::ClientRequestBuilder::new(
            url.parse().expect("invalid WS url"),
        )
        .with_header("Host", host);

        match tungstenite::client::connect(request) {
            Ok((ws_stream, _response)) => {
                self.ws = Some(ws_stream);
                self.connected = true;
                self.pending_connect = false;
                eprintln!("[WS] Connected to {}", self.addr);
                true
            }
            Err(e) => {
                eprintln!("[WS] Connect failed: {e}");
                self.pending_connect = false;
                false
            }
        }
    }

    pub fn next_event(&mut self) -> WsPacketEvent {
        if self.pending_connect || !self.connected {
            return WsPacketEvent::None;
        }

        let ws_opt = self.ws.as_mut();
        if let Some(ws) = ws_opt {
            match ws.read() {
                Ok(Message::Binary(data)) => {
                    return WsPacketEvent::Receive(data.to_vec());
                }
                Ok(Message::Text(data)) => {
                    return WsPacketEvent::Receive(data.as_bytes().to_vec());
                }
                Ok(Message::Close(_)) => {
                    self.connected = false;
                    return WsPacketEvent::Disconnect;
                }
                Ok(Message::Ping(data)) => {
                    let _ = ws.write(Message::Pong(data));
                    return WsPacketEvent::None;
                }
                Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {
                    return WsPacketEvent::None;
                }
                Err(tungstenite::Error::Io(ref e))
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    return WsPacketEvent::None;
                }
                Err(e) => {
                    eprintln!("[WS] Read error: {e}");
                    self.connected = false;
                    return WsPacketEvent::Disconnect;
                }
            }
        }

        WsPacketEvent::None
    }

    pub fn send_raw(&mut self, data: &[u8]) -> bool {
        if let Some(ref mut ws) = self.ws {
            match ws.write(Message::Binary(data.to_vec().into())) {
                Ok(()) => true,
                Err(e) => {
                    eprintln!("[WS] Send error: {e}");
                    self.connected = false;
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn send_text_raw(&mut self, data: &[u8]) -> bool {
        if let Some(ref mut ws) = self.ws {
            let text = String::from_utf8_lossy(data).to_string().into();
            match ws.write(Message::Text(text)) {
                Ok(()) => true,
                Err(e) => {
                    eprintln!("[WS] Send text error: {e}");
                    self.connected = false;
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(ref mut ws) = self.ws {
            let _ = ws.close(None);
        }
        self.connected = false;
        self.ws = None;
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn rtt(&self) -> std::time::Duration {
        std::time::Duration::from_millis(0)
    }
}
