#![cfg_attr(feature = "windows-bundle", windows_subsystem = "windows")]

#[cfg(not(target_os = "macos"))]
use dioxus::desktop::tao::window::Icon as TaoIcon;
use dioxus::{
    LaunchBuilder,
    desktop::{Config, WindowBuilder, muda::*},
};
use fenban::ui::App;

static ABOUT_ICON: &[u8] = include_bytes!("../assets/logo.png");

#[cfg(not(target_os = "macos"))]
static WINDOW_ICON: &[u8] = include_bytes!("../icons/windowicon.png");

#[cfg(not(target_os = "macos"))]
fn load_window_icon() -> Option<TaoIcon> {
    if let Ok(image) = image::load_from_memory(WINDOW_ICON) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        TaoIcon::from_rgba(rgba.into_raw(), width, height).ok()
    } else {
        None
    }
}

fn main() {
    // Custom HTML
    let index_html = r#"
        <!doctype html>
        <html>
            <head>
                <title>FenBan</title>
                <meta
                    name="viewport"
                    content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"
                />
            </head>

            <body>
                <div id="main"></div>
            </body>
        </html>
"#.to_string();

    // Custom MENU
    let menu = Menu::new();
    let home_menu = Submenu::new("主页", true);

    let about_icon = if let Ok(image) = image::load_from_memory(ABOUT_ICON) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Icon::from_rgba(rgba.into_raw(), width, height).ok()
    } else {
        None
    };

    let mut about_metadata = from_cargo_metadata!();
    about_metadata.icon = about_icon;
    about_metadata.name = Some("分班助手".to_string());
    about_metadata.copyright = Some("Copyright 2025 -- present tangxiangong".to_string());

    home_menu
        .append_items(&[
            &PredefinedMenuItem::about(Some("关于"), Some(about_metadata)),
            &MenuItem::with_id("check_update", "检查更新", true, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::hide(Some("隐藏")),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::close_window(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(Some("退出")),
        ])
        .unwrap();
    menu.append_items(&[&home_menu]).unwrap();

    let window_builder = {
        #[cfg(not(target_os = "macos"))]
        {
            WindowBuilder::new()
                .with_title("FenBan")
                .with_window_icon(load_window_icon())
        }
        #[cfg(target_os = "macos")]
        {
            WindowBuilder::new().with_title("FenBan")
        }
    };

    let config = Config::new()
        .with_custom_index(index_html)
        .with_window(window_builder)
        .with_data_directory(dirs::config_dir().unwrap().join("FenBan"))
        .with_menu(menu);
    LaunchBuilder::desktop().with_cfg(config).launch(App);
}
