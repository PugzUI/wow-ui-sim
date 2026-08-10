//! Window icon for the iced desktop app. Defers pixel rendering to the
//! always-compiled `app_icon_render` module so the same artwork backs both the
//! runtime window icon and the build-time `installer/wow-sim.ico` generator.

use iced::{Size, window};

use crate::app_icon_render::{FREEDESKTOP_APP_ID, SIZE, render_icon};

pub(super) fn settings() -> window::Settings {
    let mut settings = window::Settings {
        size: initial_window_size(),
        icon: window::icon::from_rgba(render_icon(), SIZE, SIZE).ok(),
        ..window::Settings::default()
    };
    #[cfg(target_os = "linux")]
    {
        settings.platform_specific.application_id = FREEDESKTOP_APP_ID.to_string();
    }
    settings
}

pub(super) fn initial_window_size() -> Size {
    initial_window_size_for_mode(crate::render::texture::visualizer_mode())
}

fn initial_window_size_for_mode(visualizer: bool) -> Size {
    if visualizer {
        Size::new(
            crate::render::texture::VISUALIZER_WIDTH,
            crate::render::texture::VISUALIZER_HEIGHT,
        )
    } else {
        Size::new(1024.0, 768.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_include_runtime_icon() {
        assert!(settings().icon.is_some());
    }

    #[test]
    fn visualizer_window_uses_fixed_native_stage() {
        assert_eq!(
            initial_window_size_for_mode(true),
            Size::new(2560.0, 1440.0)
        );
        assert_eq!(
            initial_window_size_for_mode(false),
            Size::new(1024.0, 768.0)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_settings_use_freedesktop_app_id() {
        assert_eq!(
            settings().platform_specific.application_id,
            FREEDESKTOP_APP_ID
        );
    }
}
