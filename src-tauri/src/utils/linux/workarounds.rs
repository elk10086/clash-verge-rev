//! Best-effort platform workarounds for known upstream issues.
//!
//! NOTE:
//! These helpers are not fixes and may stop working as environments change.

use std::env;

/// Must run before Tokio/Tauri/GTK/WebKit touch the process.
/// WebKit's renderer processes inherit these values from the parent environment.
pub fn apply_linux_webkit_workaround() {
    // WebKitGTK can abort the whole process while creating its EGL display. Avoid
    // both known failure paths: dmabuf/compositing and hardware Mesa. Every value
    // remains user-overridable by defining it before launching the application.
    set_env_default("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    set_env_default("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    set_env_default("LIBGL_ALWAYS_SOFTWARE", "1");
    set_env_default("GALLIUM_DRIVER", "llvmpipe");
    eprintln!(
        "[clash-verge] linux webkit workaround: WEBKIT_DISABLE_DMABUF_RENDERER={} WEBKIT_DISABLE_COMPOSITING_MODE={} LIBGL_ALWAYS_SOFTWARE={} GALLIUM_DRIVER={} GDK_BACKEND={}",
        env::var("WEBKIT_DISABLE_DMABUF_RENDERER").unwrap_or_default(),
        env::var("WEBKIT_DISABLE_COMPOSITING_MODE").unwrap_or_default(),
        env::var("LIBGL_ALWAYS_SOFTWARE").unwrap_or_default(),
        env::var("GALLIUM_DRIVER").unwrap_or_default(),
        env::var("GDK_BACKEND").unwrap_or_else(|_| "(auto)".into()),
    );
}

fn set_env_default(key: &str, value: &str) {
    if env::var_os(key).is_some() {
        return;
    }
    unsafe {
        env::set_var(key, value);
    }
}
