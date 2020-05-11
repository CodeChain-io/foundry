use super::*;
use ipc_channel::ipc::{channel, IpcOneShotServer, IpcReceiver, IpcSender};
use std::os::unix::net::UnixDatagram;

type Sender = IpcSender<Vec<u8>>;
type Receiver = IpcReceiver<Vec<u8>>;

pub struct ServoChannelSend(Sender);

impl IpcSend for ServoChannelSend {
    fn send(&self, data: &[u8]) {
        self.0.send(data.to_vec()).unwrap();
    }
}

pub struct Terminator(Sender);

impl Terminate for Terminator {
    fn terminate(&self) {
        self.0.send([].to_vec()).unwrap();
    }
}

pub struct ServoChannelRecv(Receiver, Sender);

impl IpcRecv for ServoChannelRecv {
    type Terminator = Terminator;

    /// Note: this function ignores timeout because it just doesn't support.
    fn recv(&self, _timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        let x = self.0.recv().unwrap();
        if x.is_empty() {
            return Err(RecvError::Termination)
        }
        Ok(x)
    }

    fn create_terminator(&self) -> Self::Terminator {
        Terminator(self.1.clone())
    }
}

pub struct ServoChannel {
    send: ServoChannelSend,
    recv: ServoChannelRecv,
}

impl IpcSend for ServoChannel {
    fn send(&self, data: &[u8]) {
        self.send.send(data)
    }
}

impl IpcRecv for ServoChannel {
    type Terminator = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        self.recv.recv(timeout)
    }

    fn create_terminator(&self) -> Self::Terminator {
        self.recv.create_terminator()
    }
}

impl Ipc for ServoChannel {
    fn arguments_for_both_ends() -> (Vec<u8>, Vec<u8>) {
        let address_server = format!("{}/{}", std::env::temp_dir().to_str().unwrap(), generate_random_name());
        let address_client = format!("{}/{}", std::env::temp_dir().to_str().unwrap(), generate_random_name());
        (
            serde_cbor::to_vec(&(true, &address_server, &address_client)).unwrap(),
            serde_cbor::to_vec(&(false, &address_client, &address_server)).unwrap(),
        )
    }

    type SendHalf = ServoChannelSend;
    type RecvHalf = ServoChannelRecv;

    fn new(data: Vec<u8>) -> Self {
        // This uses domain socket as a temporary method for the initialization.
        let (am_i_server, address_src, address_dst): (bool, String, String) = serde_cbor::from_slice(&data).unwrap();
        let socket = UnixDatagram::bind(&address_src).unwrap();
        let mut success = false;

        for _ in 0..100 {
            if socket.connect(&address_dst).is_ok() {
                success = true;
                break
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(success, "Failed to establish domain socket");
        socket.set_write_timeout(None).unwrap();
        socket.set_nonblocking(false).unwrap();

        if am_i_server {
            let (server, server_name) = IpcOneShotServer::new().unwrap();
            socket.send(&serde_cbor::to_vec(&server_name).unwrap()).unwrap();
            let (_, (recv, send, send_for_termination)): (_, (Receiver, Sender, Sender)) = server.accept().unwrap();
            ServoChannel {
                send: ServoChannelSend(send),
                recv: ServoChannelRecv(recv, send_for_termination),
            }
        } else {
            let mut buffer = [0 as u8; 129];
            let size = socket.recv(&mut buffer).unwrap();
            assert!(size <= 128);
            let client_name = serde_cbor::from_slice(&buffer[0..size]).unwrap();

            let send_ = IpcSender::connect(client_name).unwrap();
            let (send_c, recv_s) = channel().unwrap();
            let (send_s, recv_c) = channel().unwrap();
            send_.send(&(recv_s, send_s.clone(), send_c.clone())).unwrap();
            ServoChannel {
                send: ServoChannelSend(send_c),
                recv: ServoChannelRecv(recv_c, send_s),
            }
        }
    }

    fn split(self) -> (Self::SendHalf, Self::RecvHalf) {
        (self.send, self.recv)
    }
}
