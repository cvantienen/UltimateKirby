#![allow(dead_code)]
mod logging {
    pub fn write(_: &str) {}
    pub fn write_owned(_: String) {}
    pub fn live_logs(_: u64) -> String {
        "{\"next\":0,\"dropped\":0,\"entries\":[]}".into()
    }
}
mod live {
    pub mod settings_menu {
        pub fn lan_read() -> Result<(Vec<u8>, String), String> {
            let bytes = b"[yoshi.roll.grab]\nactions=grab\n".to_vec();
            let revision = kirby_assist_core::diagnostic_http::revision(&bytes);
            Ok((bytes, revision))
        }
        pub fn lan_write(_: &[u8], _: &str, allowed: impl Fn() -> bool) -> Result<(), String> {
            if allowed() {
                Ok(())
            } else {
                Err("permission revoked".into())
            }
        }
    }
}
#[path = "../../../src/diagnostics/lan.rs"]
mod lan;
