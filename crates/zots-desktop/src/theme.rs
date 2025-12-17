//! Cypherpunk dark theme for the desktop app

use iced::widget::{button, container, text, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// Custom color palette - cypherpunk aesthetics
pub mod colors {
    use iced::Color;

    // Background colors
    pub const BACKGROUND: Color = Color::from_rgb(0.02, 0.024, 0.035); // #050610
    pub const SURFACE: Color = Color::from_rgb(0.055, 0.067, 0.094); // #0e1118
    pub const SURFACE_LIGHT: Color = Color::from_rgb(0.082, 0.098, 0.133); // #151922

    // Primary colors
    pub const PRIMARY: Color = Color::from_rgb(0.388, 0.4, 0.945); // #6366f1 - indigo
    pub const PRIMARY_HOVER: Color = Color::from_rgb(0.545, 0.361, 0.965); // #8b5cf6 - violet
    pub const ACCENT: Color = Color::from_rgb(0.133, 0.773, 0.369); // #22c55e - green

    // Text colors
    pub const TEXT: Color = Color::from_rgb(0.933, 0.941, 0.957); // #eef0f4
    pub const TEXT_MUTED: Color = Color::from_rgb(0.6, 0.62, 0.66); // #999ea8
    pub const TEXT_DIM: Color = Color::from_rgb(0.4, 0.42, 0.46); // #666b75

    // Status colors
    pub const SUCCESS: Color = Color::from_rgb(0.133, 0.773, 0.369); // #22c55e
    pub const WARNING: Color = Color::from_rgb(0.961, 0.62, 0.043); // #f59e0b
    pub const ERROR: Color = Color::from_rgb(0.937, 0.267, 0.267); // #ef4444

    // Border colors
    pub const BORDER: Color = Color::from_rgb(0.2, 0.22, 0.26); // #333842
    pub const BORDER_FOCUS: Color = Color::from_rgb(0.388, 0.4, 0.945); // Primary
}

/// Get the custom dark theme
pub fn dark_theme() -> Theme {
    Theme::custom(
        "Cypherpunk".to_string(),
        iced::theme::Palette {
            background: colors::BACKGROUND,
            text: colors::TEXT,
            primary: colors::PRIMARY,
            success: colors::SUCCESS,
            danger: colors::ERROR,
        },
    )
}

/// Container styles
pub mod container_style {
    use super::*;

    pub fn surface(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }

    pub fn card(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE_LIGHT)),
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        }
    }

    pub fn sidebar(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border {
                color: colors::BORDER,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }

    pub fn header(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }

    pub fn status_bar(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Button styles
pub mod button_style {
    use super::*;

    pub fn primary(_theme: &Theme, status: button::Status) -> button::Style {
        let background = match status {
            button::Status::Active => colors::PRIMARY,
            button::Status::Hovered => colors::PRIMARY_HOVER,
            button::Status::Pressed => Color {
                a: 0.8,
                ..colors::PRIMARY
            },
            button::Status::Disabled => Color {
                a: 0.3,
                ..colors::PRIMARY
            },
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: colors::TEXT,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            shadow: Shadow {
                color: Color::from_rgba(0.388, 0.4, 0.945, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
        }
    }

    pub fn secondary(_theme: &Theme, status: button::Status) -> button::Style {
        let background = match status {
            button::Status::Active => colors::SURFACE_LIGHT,
            button::Status::Hovered => colors::BORDER,
            button::Status::Pressed => colors::SURFACE,
            button::Status::Disabled => Color {
                a: 0.3,
                ..colors::SURFACE_LIGHT
            },
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: colors::TEXT,
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }

    pub fn nav(_theme: &Theme, status: button::Status, active: bool) -> button::Style {
        let (background, text_color) = if active {
            (
                Some(Background::Color(Color {
                    a: 0.2,
                    ..colors::PRIMARY
                })),
                colors::PRIMARY,
            )
        } else {
            match status {
                button::Status::Hovered => {
                    (Some(Background::Color(colors::SURFACE_LIGHT)), colors::TEXT)
                }
                _ => (None, colors::TEXT_MUTED),
            }
        };

        button::Style {
            background,
            text_color,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn success(_theme: &Theme, status: button::Status) -> button::Style {
        let background = match status {
            button::Status::Active => colors::SUCCESS,
            button::Status::Hovered => Color {
                a: 0.8,
                ..colors::SUCCESS
            },
            button::Status::Pressed => Color {
                a: 0.6,
                ..colors::SUCCESS
            },
            button::Status::Disabled => Color {
                a: 0.3,
                ..colors::SUCCESS
            },
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: colors::TEXT,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn danger(_theme: &Theme, status: button::Status) -> button::Style {
        let background = match status {
            button::Status::Active => colors::ERROR,
            button::Status::Hovered => Color {
                a: 0.8,
                ..colors::ERROR
            },
            button::Status::Pressed => Color {
                a: 0.6,
                ..colors::ERROR
            },
            button::Status::Disabled => Color {
                a: 0.3,
                ..colors::ERROR
            },
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: colors::TEXT,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Text input styles
pub mod input_style {
    use super::*;

    pub fn default(_theme: &Theme, status: text_input::Status) -> text_input::Style {
        let border_color = match status {
            text_input::Status::Active => colors::BORDER,
            text_input::Status::Hovered => colors::TEXT_MUTED,
            text_input::Status::Focused => colors::PRIMARY,
            text_input::Status::Disabled => Color {
                a: 0.3,
                ..colors::BORDER
            },
        };

        text_input::Style {
            background: Background::Color(colors::SURFACE),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 8.0.into(),
            },
            icon: colors::TEXT_MUTED,
            placeholder: colors::TEXT_DIM,
            value: colors::TEXT,
            selection: Color {
                a: 0.3,
                ..colors::PRIMARY
            },
        }
    }
}

/// Text styles helper - returns closures for iced 0.13 API
pub mod text_style {
    use super::*;

    pub fn muted() -> impl Fn(&Theme) -> text::Style {
        move |_theme| text::Style {
            color: Some(colors::TEXT_MUTED),
        }
    }

    pub fn dim() -> impl Fn(&Theme) -> text::Style {
        move |_theme| text::Style {
            color: Some(colors::TEXT_DIM),
        }
    }

    pub fn primary() -> impl Fn(&Theme) -> text::Style {
        move |_theme| text::Style {
            color: Some(colors::PRIMARY),
        }
    }

    pub fn accent() -> impl Fn(&Theme) -> text::Style {
        move |_theme| text::Style {
            color: Some(colors::ACCENT),
        }
    }

    pub fn success() -> impl Fn(&Theme) -> text::Style {
        move |_theme| text::Style {
            color: Some(colors::SUCCESS),
        }
    }

    pub fn warning() -> impl Fn(&Theme) -> text::Style {
        move |_theme| text::Style {
            color: Some(colors::WARNING),
        }
    }

    pub fn error() -> impl Fn(&Theme) -> text::Style {
        move |_theme| text::Style {
            color: Some(colors::ERROR),
        }
    }
}
