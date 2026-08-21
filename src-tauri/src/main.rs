// Windows 发布版隐藏额外控制台，请勿删除。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    movel_lib::run()
}
