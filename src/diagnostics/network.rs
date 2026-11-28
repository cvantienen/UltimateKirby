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
    fn host_socket_errors_are_unchanged() {
        let error =
            socket_result::<()>(Err(io::Error::from(io::ErrorKind::WouldBlock))).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert_eq!(socket_result(Ok(7)).unwrap(), 7);
    }
}
