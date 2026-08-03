// 防止 debug 构建时弹出控制台窗口（Windows）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    conversation_hub_desktop_lib::run()
}
