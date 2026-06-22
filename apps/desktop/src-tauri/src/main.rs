// Prevents an extra console window on Windows in release builds. Does nothing on macOS / Linux.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    cyberos_desktop_lib::run();
}
