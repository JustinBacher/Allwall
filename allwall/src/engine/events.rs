use std::{
    io::{Read, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    time::Duration,
};

use crate::{
    cli::ipc::protocol::{Request, Response},
    engine::Engine,
    prelude::{f, info},
    sources::SourceKind,
};

impl Engine {
    pub fn handle_ipc_client(&mut self, mut stream: UnixStream) -> crate::prelude::Result<()> {
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        let mut req_buf = vec![0u8; len];
        stream.read_exact(&mut req_buf)?;

        let request: Request = bincode::deserialize(&req_buf)?;
        let response = self.handle_ipc_request(request);

        let encoded = bincode::serialize(&response)?;
        stream.write_all(&(encoded.len() as u32).to_le_bytes())?;
        stream.write_all(&encoded)?;
        stream.flush()?;
        let _ = stream.shutdown(Shutdown::Both);

        Ok(())
    }

    fn handle_ipc_request(&mut self, request: Request) -> Response {
        match request {
            Request::Next => self.handle_next(),
            Request::Prev => self.handle_prev(),
            Request::SetFps(fps) => self.handle_set_fps(fps),
        }
    }

    fn handle_next(&mut self) -> Response {
        if !matches!(self.source_kind, SourceKind::Media) {
            return Response::Error("Next command only works with media source".to_string());
        }

        if let Some(scene) = self.scenes.first_mut() {
            match scene.advance_source() {
                Ok(()) => Response::Ok,
                Err(e) => Response::Error(f!("Failed to load next image: {e}")),
            }
        } else {
            Response::Error("No scenes available".to_string())
        }
    }

    fn handle_prev(&mut self) -> Response {
        if !matches!(self.source_kind, SourceKind::Media) {
            return Response::Error("Prev command only works with media source".to_string());
        }

        if let Some(scene) = self.scenes.first_mut() {
            match scene.advance_source() {
                Ok(()) => Response::Ok,
                Err(e) => Response::Error(f!("Failed to load previous image: {e}")),
            }
        } else {
            Response::Error("No scenes available".to_string())
        }
    }

    fn handle_set_fps(&mut self, fps: u32) -> Response {
        if fps == 0 {
            return Response::Error("FPS must be greater than 0".to_string());
        }
        self.fps = fps as f32;
        info!("FPS set to {}", fps);
        Response::Ok
    }
}
