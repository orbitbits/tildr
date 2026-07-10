pub fn has_display() -> bool {
  #[cfg(target_os = "macos")]
  return true; // macOS sempre tem GUI quando rodando localmente

  // #[cfg(target_os = "windows")]
  // return true;

  #[cfg(all(unix, not(target_os = "macos")))]
  return std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();

  #[cfg(not(any(unix, windows)))]
  return false;
}
