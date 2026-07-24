//! Best-effort platform workarounds for known upstream issues.
//!
//! NOTE:
//! These helpers are not fixes and may stop working as environments change.

use std::env;
use std::ffi::CString;

/// Must run before Tokio/Tauri/GTK/WebKit touch the process.
/// Uses libc `setenv` so WebKit's separate renderer process inherits the values.
pub fn apply_linux_webkit_workaround() {
    // Blank WebView on Linux is commonly caused by WebKitGTK dmabuf/EGL paths.
    set_env_default("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    set_env_default("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    // AppImage / some DEs still fail under Wayland EGL; prefer X11 backend when unset.
    if env::var_os("WAYLAND_DISPLAY").is_some() && env::var_os("GDK_BACKEND").is_none() {
        set_env_default("GDK_BACKEND", "x11");
    }

    eprintln!(
        "[clash-verge] linux webkit workaround: WEBKIT_DISABLE_DMABUF_RENDERER={} WEBKIT_DISABLE_COMPOSITING_MODE={} GDK_BACKEND={}",
        env::var("WEBKIT_DISABLE_DMABUF_RENDERER").unwrap_or_default(),
        env::var("WEBKIT_DISABLE_COMPOSITING_MODE").unwrap_or_default(),
        env::var("GDK_BACKEND").unwrap_or_else(|_| "(unset)".into()),
    );
}

fn set_env_default(key: &str, value: &str) {
    if env::var_os(key).is_some() {
        return;
    }
    // Keep Rust's env map in sync.
    unsafe {
        env::set_var(key, value);
    }
    // Also push into the C environ for forked WebKit/GTK helpers.
    let Ok(c_key) = CString::new(key) else {
        return;
    };
    let Ok(c_val) = CString::new(value) else {
        return;
    };
    unsafe {
        libc::setenv(c_key.as_ptr(), c_val.as_ptr(), 1);
    }
}
