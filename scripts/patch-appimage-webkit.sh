#!/usr/bin/env bash
# Patch a Tauri AppImage so WebKit starts without a blank window on Linux hosts.
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "usage: $0 <appimage> [appimage...]" >&2
  exit 1
fi

if [ ! -x /tmp/appimagetool ]; then
  curl -fsSL -o /tmp/appimagetool \
    https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
  chmod +x /tmp/appimagetool
fi

patch_one() {
  local appimage="$1"
  echo "Patching AppImage: $appimage"
  chmod +x "$appimage"

  local work
  work="$(mktemp -d)"
  cleanup() { rm -rf "$work"; }
  trap cleanup RETURN

  (
    cd "$work"
    "$appimage" --appimage-extract

    python3 - <<'PY'
from pathlib import Path

path = Path("squashfs-root/AppRun")
if not path.exists():
    raise SystemExit("AppRun missing")

text = path.read_text()
marker = "WEBKIT_DISABLE_DMABUF_RENDERER"
inject = """export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
export WEBKIT_DISABLE_COMPOSITING_MODE="${WEBKIT_DISABLE_COMPOSITING_MODE:-1}"
export LIBGL_ALWAYS_SOFTWARE="${LIBGL_ALWAYS_SOFTWARE:-1}"
export GALLIUM_DRIVER="${GALLIUM_DRIVER:-llvmpipe}"
if [ -n "${WAYLAND_DISPLAY:-}" ] && [ "${GDK_BACKEND:-}" = "x11" ]; then
  export GDK_BACKEND=wayland,x11
fi

"""

if marker in text:
    print("AppRun already patched")
elif text.startswith("#!"):
    nl = text.find("\n")
    path.write_text(text[: nl + 1] + inject + text[nl + 1 :])
    print("Patched AppRun shebang script")
else:
    wrapper = Path("squashfs-root/AppRun.wrapped")
    path.rename(wrapper)
    path.write_text("#!/bin/sh\n" + inject + 'exec "$(dirname "$0")/AppRun.wrapped" "$@"\n')
    path.chmod(0o755)
    print("Wrapped binary AppRun")
PY

    find squashfs-root -type f \( \
      -name 'libEGL.so*' -o \
      -name 'libEGL_mesa.so*' -o \
      -name 'libGLESv2.so*' -o \
      -name 'libGLX.so*' -o \
      -name 'libGLX_mesa.so*' \
    \) -print -delete || true

    rm -f "$appimage"
    ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 /tmp/appimagetool squashfs-root "$appimage"
    chmod +x "$appimage"
  )

  echo "Patched: $appimage"
}

for appimage in "$@"; do
  patch_one "$(realpath "$appimage")"
done
