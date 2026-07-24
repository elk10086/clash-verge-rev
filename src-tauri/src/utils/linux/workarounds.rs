//! Best-effort platform workarounds for known upstream issues.
//!
//! NOTE:
//! These helpers are not fixes and may stop working as environments change.

use clash_verge_logging::{Type, logging};
use std::env;

fn set_env_default(key: &str, value: &str) -> bool {
    if env::var_os(key).is_some() {
        return false;
    }
    unsafe {
        env::set_var(key, value);
    }
    true
}

/// Linux WebKitGTK blank windows are common across AppImage/deb, NVIDIA, and
/// Wayland. Apply the safe renderer path by default so users can open the app
/// without setting env vars manually. Existing env values are left untouched.
pub fn apply_linux_webkit_workaround() {
    let mut changed = false;
    changed |= set_env_default("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    changed |= set_env_default("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    if changed {
        logging!(
            info,
            Type::Setup,
            "Applied Linux WebKit safe renderer: WEBKIT_DISABLE_DMABUF_RENDERER=1, WEBKIT_DISABLE_COMPOSITING_MODE=1"
        );
    }
}
