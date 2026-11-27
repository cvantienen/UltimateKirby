#![allow(unexpected_cfgs)]
use std::io;

pub(crate) fn socket_result<T>(result: io::Result<T>) -> io::Result<T> {
    result.map_err(|error| {
        #[cfg(target_os = "switch")]
        {
            // Switch std converts native -1 returns to error 1 and omits errno decoding.
            let code = unsafe { nnsdk::nn::socket::GetLastError() } as i32;
            if code != 0 {
                return native_error(code);
            }
        }
        error
    })
}

pub(crate) struct Stream<'a, F>(pub(crate) &'a mut std::net::TcpStream, pub(crate) F);

fn permission(allowed: impl Fn() -> bool) -> io::Result<()> {
    if allowed() {
        Ok(())
    } else {
        Err(io::ErrorKind::PermissionDenied.into())
    }
}

impl<F: Fn() -> bool> io::Read for Stream<'_, F> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        permission(&self.1)?;
        let result = socket_result(io::Read::read(self.0, buffer));
        permission(&self.1)?;
        result
    }
}

impl<F: Fn() -> bool> io::Write for Stream<'_, F> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        permission(&self.1)?;
        socket_result(io::Write::write(self.0, buffer))
    }

    fn flush(&mut self) -> io::Result<()> {
        permission(&self.1)?;
        socket_result(io::Write::flush(self.0))
    }
}

#[cfg(any(target_os = "switch", test))]
fn native_error(code: i32) -> io::Error {
    // libc-nnsdk uses musl errno numbering, not the host macOS numbering.
    let kind = match code {
        4 => io::ErrorKind::Interrupted,
        11 => io::ErrorKind::WouldBlock,
        110 => io::ErrorKind::TimedOut,
        111 => io::ErrorKind::ConnectionRefused,
        _ => io::ErrorKind::Other,
    };
    io::Error::new(kind, format!("Switch socket errno {code}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_socket_errors_preserve_retry_classification_and_actual_code() {
        for (code, kind) in [
            (4, io::ErrorKind::Interrupted),
            (11, io::ErrorKind::WouldBlock),
            (110, io::ErrorKind::TimedOut),
            (111, io::ErrorKind::ConnectionRefused),
            (99, io::ErrorKind::Other),
        ] {
            let error = native_error(code);
            assert_eq!(error.kind(), kind);
            assert!(error.to_string().contains(&code.to_string()));
        }
    }

    #[test]
    fn revoked_stream_neither_reads_nor_writes() {
        use std::io::{Read, Write};
        use std::net::{TcpListener, TcpStream};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut peer = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (mut socket, _) = listener.accept().unwrap();
        peer.write_all(b"request").unwrap();
        let mut bytes = [0; 7];
        {
            let mut guarded = Stream(&mut socket, || false);
            assert_eq!(
                guarded.read(&mut bytes).unwrap_err().kind(),
                io::ErrorKind::PermissionDenied
            );
            assert_eq!(
                guarded.write(b"response").unwrap_err().kind(),
                io::ErrorKind::PermissionDenied
            );
            assert_eq!(
                guarded.flush().unwrap_err().kind(),
                io::ErrorKind::PermissionDenied
            );
        }
        socket.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"request");
        peer.set_nonblocking(true).unwrap();
        assert_eq!(
            peer.read(&mut bytes).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
    }

    #[test]
    fn host_socket_errors_are_unchanged() {
        let error =
            socket_result::<()>(Err(io::Error::from(io::ErrorKind::WouldBlock))).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert_eq!(socket_result(Ok(7)).unwrap(), 7);
    }
}
