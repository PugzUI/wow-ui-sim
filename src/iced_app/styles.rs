//! UI style functions for iced widgets.

use iced::widget::{button, pick_list, text_input};
use iced::{Border, Color, Theme};

// WoW-inspired color palette
pub mod palette {
    use iced::Color;

    pub const CHROME_BACKGROUND: Color = Color::BLACK;
    pub const CHROME_BORDER: Color = Color::from_rgb(69.0 / 255.0, 69.0 / 255.0, 69.0 / 255.0);
    pub const CHROME_TEXT_DIM: Color = Color::from_rgb(196.0 / 255.0, 196.0 / 255.0, 196.0 / 255.0);
    pub const CHROME_TEXT: Color = Color::from_rgb(244.0 / 255.0, 244.0 / 255.0, 244.0 / 255.0);
    pub const HEADER_GOLD: Color = Color::from_rgb(247.0 / 255.0, 196.0 / 255.0, 0.0);
    pub const RUN_GOLD: Color = Color::from_rgb(255.0 / 255.0, 196.0 / 255.0, 0.0);

    // Keep the simulator shell around the WoW stage on the same near-black
    // surface as the command strip. The stage itself still owns its rendered
    // background inside the canvas.
    pub const BG_DARK: Color = CHROME_BACKGROUND;
    pub const BG_PANEL: Color = Color::from_rgb(0.12, 0.12, 0.14);
    pub const BG_INPUT: Color = Color::from_rgb(0.06, 0.06, 0.08);
    pub const GOLD: Color = Color::from_rgb(0.85, 0.65, 0.13);
    pub const GOLD_DIM: Color = Color::from_rgb(0.55, 0.42, 0.10);
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.90, 0.85);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.60, 0.58, 0.55);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.45, 0.43, 0.40);
    pub const BORDER: Color = Color::from_rgb(0.25, 0.23, 0.20);
    pub const BORDER_HIGHLIGHT: Color = Color::from_rgb(0.40, 0.35, 0.25);
    pub const CONSOLE_TEXT: Color = Color::from_rgb(0.70, 0.85, 0.70);
}

/// Style for event buttons (ADDON_LOADED, PLAYER_LOGIN, etc.).
pub fn event_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Active => (palette::BG_PANEL, palette::TEXT_SECONDARY),
        button::Status::Hovered => (palette::BORDER_HIGHLIGHT, palette::GOLD),
        button::Status::Pressed => (palette::GOLD_DIM, palette::TEXT_PRIMARY),
        button::Status::Disabled => (palette::BG_DARK, palette::TEXT_MUTED),
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color,
        border: Border {
            color: palette::BORDER,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the command-strip Options button.
pub fn command_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let text_color = match status {
        button::Status::Hovered | button::Status::Pressed => palette::CHROME_TEXT,
        button::Status::Active | button::Status::Disabled => palette::CHROME_TEXT_DIM,
    };

    button::Style {
        background: Some(iced::Background::Color(palette::CHROME_BACKGROUND)),
        text_color,
        border: Border {
            color: palette::CHROME_BORDER,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the Run button.
pub fn run_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color, border_color) = match status {
        button::Status::Active => (palette::RUN_GOLD, Color::BLACK, palette::CHROME_BORDER),
        button::Status::Hovered => (palette::RUN_GOLD, Color::BLACK, palette::CHROME_BORDER),
        button::Status::Pressed => (palette::RUN_GOLD, Color::BLACK, palette::CHROME_BORDER),
        button::Status::Disabled => (
            palette::CHROME_BACKGROUND,
            palette::CHROME_TEXT_DIM,
            palette::CHROME_BORDER,
        ),
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the command input field.
pub fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Active => palette::CHROME_BORDER,
        text_input::Status::Hovered => palette::CHROME_BORDER,
        text_input::Status::Focused { is_hovered: _ } => palette::CHROME_BORDER,
        text_input::Status::Disabled => palette::CHROME_BORDER,
    };

    text_input::Style {
        background: iced::Background::Color(palette::CHROME_BACKGROUND),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 3.0.into(),
        },
        icon: palette::CHROME_TEXT_DIM,
        placeholder: palette::CHROME_TEXT_DIM,
        value: palette::CHROME_TEXT,
        selection: palette::HEADER_GOLD,
    }
}

/// Style for pick_list dropdowns (class/race selectors).
pub fn pick_list_style(_theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let border_color = match status {
        pick_list::Status::Active => palette::BORDER,
        pick_list::Status::Hovered => palette::BORDER_HIGHLIGHT,
        pick_list::Status::Opened { .. } => palette::GOLD_DIM,
    };

    pick_list::Style {
        text_color: palette::TEXT_PRIMARY,
        placeholder_color: palette::TEXT_MUTED,
        handle_color: palette::TEXT_SECONDARY,
        background: iced::Background::Color(palette::BG_INPUT),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 3.0.into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::palette;
    use iced::Color;
    use iced::Theme;
    use iced::widget::{button, text_input};

    #[test]
    fn command_chrome_palette_matches_design() {
        assert_eq!(palette::CHROME_BACKGROUND, Color::BLACK);
        assert_eq!(
            palette::CHROME_BORDER,
            Color::from_rgb(69.0 / 255.0, 69.0 / 255.0, 69.0 / 255.0)
        );
        assert_eq!(
            palette::CHROME_TEXT_DIM,
            Color::from_rgb(196.0 / 255.0, 196.0 / 255.0, 196.0 / 255.0)
        );
        assert_eq!(
            palette::CHROME_TEXT,
            Color::from_rgb(244.0 / 255.0, 244.0 / 255.0, 244.0 / 255.0)
        );
        assert_eq!(
            palette::HEADER_GOLD,
            Color::from_rgb(247.0 / 255.0, 196.0 / 255.0, 0.0)
        );
        assert_eq!(
            palette::RUN_GOLD,
            Color::from_rgb(255.0 / 255.0, 196.0 / 255.0, 0.0)
        );
    }

    #[test]
    fn run_button_uses_gold_background_and_black_text() {
        let style = super::run_button_style(&Theme::Dark, button::Status::Active);

        assert_eq!(
            style.background,
            Some(iced::Background::Color(palette::RUN_GOLD))
        );
        assert_eq!(style.text_color, Color::BLACK);
        assert_eq!(style.border.color, palette::CHROME_BORDER);
    }

    #[test]
    fn command_controls_use_chrome_background_border_and_text() {
        let options = super::command_button_style(&Theme::Dark, button::Status::Active);
        assert_eq!(
            options.background,
            Some(iced::Background::Color(palette::CHROME_BACKGROUND))
        );
        assert_eq!(options.text_color, palette::CHROME_TEXT_DIM);
        assert_eq!(options.border.color, palette::CHROME_BORDER);

        let input = super::input_style(&Theme::Dark, text_input::Status::Active);
        assert_eq!(
            input.background,
            iced::Background::Color(palette::CHROME_BACKGROUND)
        );
        assert_eq!(input.border.color, palette::CHROME_BORDER);
        assert_eq!(input.placeholder, palette::CHROME_TEXT_DIM);
        assert_eq!(input.value, palette::CHROME_TEXT);
    }
}
