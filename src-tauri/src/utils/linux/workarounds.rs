//! Best-effort platform workarounds for known upstream issues.
//!
//! NOTE:
//! These helpers are not fixes and may stop working as environments change.

use clash_verge_logging::{Type, logging};
use std::{env, fs, path::Path, process::Command};

fn set_env_default(key: &str, value: &str) -> bool {
    if env::var_os(key).is_some() {
        return false;
    }
    unsafe {
        env::set_var(key, value);
    }
    true
}

fn enable_webkit_safe_renderer(reason: &str) {
    let mut changed = false;
    // Blank WebView on Linux is commonly caused by WebKitGTK dmabuf/EGL paths.
    changed |= set_env_default("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    changed |= set_env_default("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    if changed {
        logging!(
            info,
            Type::Setup,
            "Applied WebKit safe renderer ({reason}): WEBKIT_DISABLE_DMABUF_RENDERER=1, WEBKIT_DISABLE_COMPOSITING_MODE=1"
        );
    }
}

pub fn apply_nvidia_dmabuf_renderer_workaround() {
    if has_nvidia_gpu() {
        enable_webkit_safe_renderer("NVIDIA GPU");
    }
}

/// AppImage + WebKitGTK frequently hits EGL_BAD_PARAMETER / blank windows on
/// mixed GPU stacks; prefer the safer software path unless the user overrides.
pub fn apply_appimage_webkit_workaround() {
    if env::var_os("APPIMAGE").is_none() {
        return;
    }
    enable_webkit_safe_renderer("AppImage");
}

/// !Might cause more memory footpoint
pub fn apply_wayland_webkit_fix() {
    let is_wayland = env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland";

    if !is_wayland {
        return;
    }

    let version = Command::new("pkg-config")
        .args(["--modversion", "wayland-client"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok());

    // On Wayland, prefer the safe path broadly: version checks alone miss many
    // blank-window cases (NVIDIA, hybrid GPU, nested compositors).
    if version.is_some_and(|v| v.trim() <= "1.23.0") || has_nvidia_gpu() {
        enable_webkit_safe_renderer("Wayland");
    }
}

fn has_nvidia_gpu() -> bool {
    if Path::new("/proc/driver/nvidia/version").exists()
        || Path::new("/sys/module/nvidia").exists()
        || Path::new("/sys/module/nvidia_drm").exists()
    {
        return true;
    }

    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return false;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }

        let vendor_path = entry.path().join("device/vendor");
        let Ok(vendor) = fs::read_to_string(vendor_path) else {
            continue;
        };
        if vendor.trim().eq_ignore_ascii_case("0x10de") {
            return true;
        }
    }

    false
}
