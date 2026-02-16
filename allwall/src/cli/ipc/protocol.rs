use std::{
    env,
    io::{self, Read, Write},
    os::unix::net::{SocketAddr, UnixStream},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{cli::error::CliError, prelude::*};

const SOCKET_NAME: &str = "allwall.sock";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    Next,
    Prev,
    SetFps(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Ok,
    Error(String),
}

impl Response {
    pub fn is_ok(&self) -> bool {
        matches!(self, Response::Ok)
    }
}

pub fn socket_path() -> PathBuf {
    let xdg = xdg::BaseDirectories::with_prefix("allwall");
    xdg.get_runtime_directory().map(|p| p.join(SOCKET_NAME)).unwrap_or_else(|_| {
        let user = env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        PathBuf::from(format!("/tmp/allwall-{user}.sock"))
    })
}

pub fn socket_addr() -> Result<SocketAddr> {
    SocketAddr::from_pathname(socket_path()).map_err(|_| CliError::SocketAddrCreate.into())
}

pub fn is_daemon_running() -> bool {
    let path = socket_path();
    if !path.exists() {
        return false;
    }

    UnixStream::connect(&path).is_ok()
}

pub fn send_request(request: &Request) -> Result<Response> {
    let path = socket_path();
    let mut stream = UnixStream::connect(&path).map_err(|e| match e.kind() {
        io::ErrorKind::ConnectionRefused => Error::DaemonNotRunning,
        _ => Error::from(e),
    })?;

    let encoded = bincode::serialize(request)?;
    stream.write_all(&(encoded.len() as u32).to_le_bytes())?;
    stream.write_all(&encoded)?;
    stream.flush()?;

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;

    let mut resp_buf = vec![0u8; len];
    stream.read_exact(&mut resp_buf)?;

    let response: Response = bincode::deserialize(&resp_buf)?;
    Ok(response)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialize_next() {
        let request = Request::Next;
        let encoded = bincode::serialize(&request).unwrap();
        let decoded: Request = bincode::deserialize(&encoded).unwrap();
        assert!(matches!(decoded, Request::Next));
    }

    #[test]
    fn test_request_serialize_prev() {
        let request = Request::Prev;
        let encoded = bincode::serialize(&request).unwrap();
        let decoded: Request = bincode::deserialize(&encoded).unwrap();
        assert!(matches!(decoded, Request::Prev));
    }

    #[test]
    fn test_request_serialize_set_fps() {
        let request = Request::SetFps(60);
        let encoded = bincode::serialize(&request).unwrap();
        let decoded: Request = bincode::deserialize(&encoded).unwrap();
        match decoded {
            Request::SetFps(fps) => assert_eq!(fps, 60),
            _ => panic!("Expected SetFps variant"),
        }
    }

    #[test]
    fn test_response_ok_is_ok() {
        let response = Response::Ok;
        assert!(response.is_ok());
    }

    #[test]
    fn test_response_error_is_not_ok() {
        let response = Response::Error("test error".to_string());
        assert!(!response.is_ok());
    }

    #[test]
    fn test_response_serialize_ok() {
        let response = Response::Ok;
        let encoded = bincode::serialize(&response).unwrap();
        let decoded: Response = bincode::deserialize(&encoded).unwrap();
        assert!(decoded.is_ok());
    }

    #[test]
    fn test_response_serialize_error() {
        let response = Response::Error("daemon error".to_string());
        let encoded = bincode::serialize(&response).unwrap();
        let decoded: Response = bincode::deserialize(&encoded).unwrap();
        match decoded {
            Response::Error(msg) => assert_eq!(msg, "daemon error"),
            Response::Ok => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_socket_path_contains_socket_name() {
        let path = socket_path();
        assert!(path.to_string_lossy().contains("allwall.sock"));
    }
}
