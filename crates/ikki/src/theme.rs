//! Ikki Design System
//!
//! A modern, clean design system inspired by consumer fintech apps.
//! Light theme with subtle shadows and rounded corners.

use iced::widget::{button, container, scrollable, text, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// Design tokens - Colors
pub mod colors {
    use iced::Color;

    // Background colors - Clean whites and subtle grays
    pub const BACKGROUND: Color = Color::from_rgb(0.976, 0.976, 0.98); // #f9f9fa
    pub const SURFACE: Color = Color::WHITE; // #ffffff
    pub const SURFACE_SECONDARY: Color = Color::from_rgb(0.96, 0.96, 0.965); // #f5f5f7
    pub const SURFACE_ELEVATED: Color = Color::WHITE;

    // Primary brand colors - Deep indigo for trust
    pub const PRIMARY: Color = Color::from_rgb(0.227, 0.227, 0.392); // #3a3a64
    pub const PRIMARY_LIGHT: Color = Color::from_rgb(0.319, 0.319, 0.502); // #515180
    pub const PRIMARY_DARK: Color = Color::from_rgb(0.157, 0.157, 0.294); // #28284b

    // Accent - Zcash gold/amber
    pub const ACCENT: Color = Color::from_rgb(0.957, 0.718, 0.157); // #f4b728
    pub const ACCENT_LIGHT: Color = Color::from_rgb(0.973, 0.792, 0.361); // #f8ca5c

    // Action colors
    pub const SEND: Color = Color::from_rgb(0.937, 0.325, 0.314); // #ef5350 - warm red
    pub const RECEIVE: Color = Color::from_rgb(0.298, 0.686, 0.314); // #4caf50 - green

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.118, 0.118, 0.145); // #1e1e25
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.475, 0.475, 0.525); // #797986
    pub const TEXT_TERTIARY: Color = Color::from_rgb(0.651, 0.651, 0.686); // #a6a6af
    pub const TEXT_ON_DARK: Color = Color::WHITE;
    pub const TEXT_ON_ACCENT: Color = Color::from_rgb(0.157, 0.157, 0.294);

    // Status colors
    pub const SUCCESS: Color = Color::from_rgb(0.298, 0.686, 0.314); // #4caf50
    pub const WARNING: Color = Color::from_rgb(1.0, 0.596, 0.0); // #ff9800
    pub const ERROR: Color = Color::from_rgb(0.898, 0.224, 0.208); // #e53935
    pub const INFO: Color = Color::from_rgb(0.129, 0.588, 0.953); // #2196f3

    // Border & Divider
    pub const BORDER: Color = Color::from_rgb(0.886, 0.886, 0.902); // #e2e2e6
    pub const BORDER_LIGHT: Color = Color::from_rgb(0.933, 0.933, 0.941); // #eeeff0
    pub const DIVIDER: Color = Color::from_rgb(0.933, 0.933, 0.941); // #eeeff0

    // Card gradients for account cards
    pub const CARD_GRADIENT_START: Color = Color::from_rgb(0.227, 0.227, 0.392); // #3a3a64
    pub const CARD_GRADIENT_END: Color = Color::from_rgb(0.319, 0.275, 0.502); // #514680

    // Shielded indicator - subtle purple
    pub const SHIELDED: Color = Color::from_rgb(0.608, 0.319, 0.878); // #9b51e0
}

/// Spacing scale (in pixels)
pub mod spacing {
    pub const XXXS: f32 = 2.0;
    pub const XXS: f32 = 4.0;
    pub const XS: f32 = 8.0;
    pub const SM: f32 = 12.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 32.0;
    pub const XXL: f32 = 48.0;
    pub const XXXL: f32 = 64.0;
}

/// Border radius scale
pub mod radius {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const FULL: f32 = 9999.0;
}

/// Typography scale
pub mod typography {
    pub const DISPLAY: f32 = 48.0;
    pub const H1: f32 = 32.0;
    pub const H2: f32 = 24.0;
    pub const H3: f32 = 20.0;
    pub const BODY_LARGE: f32 = 18.0;
    pub const BODY: f32 = 16.0;
    pub const BODY_SMALL: f32 = 14.0;
    pub const CAPTION: f32 = 12.0;
    pub const OVERLINE: f32 = 10.0;
}

/// Shadow definitions
pub mod shadows {
    use super::*;

    pub fn sm() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.04),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        }
    }

    pub fn md() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        }
    }

    pub fn lg() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        }
    }

    pub fn card() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 32.0,
        }
    }

    pub fn glow(color: Color) -> Shadow {
        Shadow {
            color: Color { a: 0.3, ..color },
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        }
    }
}

/// Get the Ikki light theme
pub fn ikki_theme() -> Theme {
    Theme::custom(
        "Ikki".to_string(),
        iced::theme::Palette {
            background: colors::BACKGROUND,
            text: colors::TEXT_PRIMARY,
            primary: colors::PRIMARY,
            success: colors::SUCCESS,
            danger: colors::ERROR,
        },
    )
}

/// Container styles
pub mod container_style {
    use super::*;

    pub fn transparent(_theme: &Theme) -> container::Style {
        container::Style::default()
    }

    pub fn background(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND)),
            ..Default::default()
        }
    }

    pub fn surface(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border {
                color: colors::BORDER_LIGHT,
                width: 1.0,
                radius: radius::MD.into(),
            },
            shadow: shadows::sm(),
            ..Default::default()
        }
    }

    pub fn card(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border {
                color: colors::BORDER_LIGHT,
                width: 1.0,
                radius: radius::LG.into(),
            },
            shadow: shadows::md(),
            ..Default::default()
        }
    }

    pub fn card_elevated(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE_ELEVATED)),
            border: Border {
                radius: radius::XL.into(),
                ..Default::default()
            },
            shadow: shadows::lg(),
            ..Default::default()
        }
    }

    pub fn account_card(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::CARD_GRADIENT_START)),
            border: Border {
                radius: radius::XL.into(),
                ..Default::default()
            },
            shadow: shadows::card(),
            ..Default::default()
        }
    }

    pub fn nav_item(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn nav_item_active(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE_SECONDARY)),
            border: Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn pill(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE_SECONDARY)),
            border: Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn success_pill(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color {
                a: 0.1,
                ..colors::SUCCESS
            })),
            border: Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn warning_pill(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color {
                a: 0.1,
                ..colors::WARNING
            })),
            border: Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn modal_backdrop(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
            ..Default::default()
        }
    }

    pub fn header(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border {
                color: colors::BORDER_LIGHT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: shadows::sm(),
            ..Default::default()
        }
    }

    pub fn sidebar(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border {
                color: colors::BORDER_LIGHT,
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
        let (background, text_color) = match status {
            button::Status::Active => (colors::PRIMARY, colors::TEXT_ON_DARK),
            button::Status::Hovered => (colors::PRIMARY_LIGHT, colors::TEXT_ON_DARK),
            button::Status::Pressed => (colors::PRIMARY_DARK, colors::TEXT_ON_DARK),
            button::Status::Disabled => (
                Color {
                    a: 0.4,
                    ..colors::PRIMARY
                },
                Color {
                    a: 0.6,
                    ..colors::TEXT_ON_DARK
                },
            ),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            shadow: shadows::sm(),
        }
    }

    pub fn primary_large(_theme: &Theme, status: button::Status) -> button::Style {
        let mut style = primary(_theme, status);
        style.border.radius = radius::LG.into();
        style.shadow = shadows::md();
        style
    }

    pub fn accent(_theme: &Theme, status: button::Status) -> button::Style {
        let (background, text_color) = match status {
            button::Status::Active => (colors::ACCENT, colors::TEXT_ON_ACCENT),
            button::Status::Hovered => (colors::ACCENT_LIGHT, colors::TEXT_ON_ACCENT),
            button::Status::Pressed => (colors::ACCENT, colors::TEXT_ON_ACCENT),
            button::Status::Disabled => (
                Color {
                    a: 0.4,
                    ..colors::ACCENT
                },
                Color {
                    a: 0.6,
                    ..colors::TEXT_ON_ACCENT
                },
            ),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            shadow: shadows::glow(colors::ACCENT),
        }
    }

    pub fn secondary(_theme: &Theme, status: button::Status) -> button::Style {
        let (background, border_color) = match status {
            button::Status::Active => (colors::SURFACE, colors::BORDER),
            button::Status::Hovered => (colors::SURFACE_SECONDARY, colors::BORDER),
            button::Status::Pressed => (colors::SURFACE_SECONDARY, colors::PRIMARY),
            button::Status::Disabled => (colors::SURFACE_SECONDARY, colors::BORDER_LIGHT),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: radius::MD.into(),
            },
            shadow: Shadow::default(),
        }
    }

    pub fn ghost(_theme: &Theme, status: button::Status) -> button::Style {
        let (background, text_color) = match status {
            button::Status::Active => (Color::TRANSPARENT, colors::TEXT_SECONDARY),
            button::Status::Hovered => (colors::SURFACE_SECONDARY, colors::TEXT_PRIMARY),
            button::Status::Pressed => (colors::SURFACE_SECONDARY, colors::PRIMARY),
            button::Status::Disabled => (
                Color::TRANSPARENT,
                Color {
                    a: 0.4,
                    ..colors::TEXT_SECONDARY
                },
            ),
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn send(_theme: &Theme, status: button::Status) -> button::Style {
        let background = match status {
            button::Status::Active => colors::SEND,
            button::Status::Hovered => Color {
                a: 0.9,
                ..colors::SEND
            },
            button::Status::Pressed => Color {
                a: 0.8,
                ..colors::SEND
            },
            button::Status::Disabled => Color {
                a: 0.4,
                ..colors::SEND
            },
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: colors::TEXT_ON_DARK,
            border: Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            shadow: shadows::glow(colors::SEND),
        }
    }

    pub fn receive(_theme: &Theme, status: button::Status) -> button::Style {
        let background = match status {
            button::Status::Active => colors::RECEIVE,
            button::Status::Hovered => Color {
                a: 0.9,
                ..colors::RECEIVE
            },
            button::Status::Pressed => Color {
                a: 0.8,
                ..colors::RECEIVE
            },
            button::Status::Disabled => Color {
                a: 0.4,
                ..colors::RECEIVE
            },
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: colors::TEXT_ON_DARK,
            border: Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            shadow: shadows::glow(colors::RECEIVE),
        }
    }

    pub fn icon_circle(_theme: &Theme, status: button::Status) -> button::Style {
        let background = match status {
            button::Status::Active => colors::SURFACE_SECONDARY,
            button::Status::Hovered => colors::BORDER,
            button::Status::Pressed => colors::BORDER,
            button::Status::Disabled => colors::SURFACE_SECONDARY,
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn nav(_theme: &Theme, status: button::Status, active: bool) -> button::Style {
        let (background, text_color) = if active {
            (
                Some(Background::Color(colors::SURFACE_SECONDARY)),
                colors::PRIMARY,
            )
        } else {
            match status {
                button::Status::Hovered => (
                    Some(Background::Color(colors::SURFACE_SECONDARY)),
                    colors::TEXT_PRIMARY,
                ),
                _ => (None, colors::TEXT_SECONDARY),
            }
        };

        button::Style {
            background,
            text_color,
            border: Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn danger(_theme: &Theme, status: button::Status) -> button::Style {
        let background = match status {
            button::Status::Active => colors::ERROR,
            button::Status::Hovered => Color {
                a: 0.9,
                ..colors::ERROR
            },
            button::Status::Pressed => Color {
                a: 0.8,
                ..colors::ERROR
            },
            button::Status::Disabled => Color {
                a: 0.4,
                ..colors::ERROR
            },
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: colors::TEXT_ON_DARK,
            border: Border {
                radius: radius::MD.into(),
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
        let (border_color, background) = match status {
            text_input::Status::Active => (colors::BORDER, colors::SURFACE),
            text_input::Status::Hovered => (colors::TEXT_TERTIARY, colors::SURFACE),
            text_input::Status::Focused => (colors::PRIMARY, colors::SURFACE),
            text_input::Status::Disabled => (colors::BORDER_LIGHT, colors::SURFACE_SECONDARY),
        };

        text_input::Style {
            background: Background::Color(background),
            border: Border {
                color: border_color,
                width: 1.5,
                radius: radius::MD.into(),
            },
            icon: colors::TEXT_TERTIARY,
            placeholder: colors::TEXT_TERTIARY,
            value: colors::TEXT_PRIMARY,
            selection: Color {
                a: 0.2,
                ..colors::PRIMARY
            },
        }
    }

    pub fn large(_theme: &Theme, status: text_input::Status) -> text_input::Style {
        let mut style = default(_theme, status);
        style.border.radius = radius::LG.into();
        style
    }

    pub fn amount(_theme: &Theme, status: text_input::Status) -> text_input::Style {
        let border_color = match status {
            text_input::Status::Active => Color::TRANSPARENT,
            text_input::Status::Hovered => Color::TRANSPARENT,
            text_input::Status::Focused => colors::PRIMARY,
            text_input::Status::Disabled => Color::TRANSPARENT,
        };

        text_input::Style {
            background: Background::Color(Color::TRANSPARENT),
            border: Border {
                color: border_color,
                width: 0.0,
                radius: 0.0.into(),
            },
            icon: colors::TEXT_TERTIARY,
            placeholder: colors::TEXT_TERTIARY,
            value: colors::TEXT_PRIMARY,
            selection: Color {
                a: 0.2,
                ..colors::PRIMARY
            },
        }
    }
}

/// Scrollable styles
pub mod scrollable_style {
    use super::*;

    pub fn default(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
        let _ = theme;
        let scrollbar_color = match status {
            scrollable::Status::Active => Color::TRANSPARENT,
            scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => {
                colors::BORDER
            }
        };

        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: scrollable::Rail {
                background: None,
                border: Border::default(),
                scroller: scrollable::Scroller {
                    color: scrollbar_color,
                    border: Border {
                        radius: radius::FULL.into(),
                        ..Default::default()
                    },
                },
            },
            horizontal_rail: scrollable::Rail {
                background: None,
                border: Border::default(),
                scroller: scrollable::Scroller {
                    color: scrollbar_color,
                    border: Border {
                        radius: radius::FULL.into(),
                        ..Default::default()
                    },
                },
            },
            gap: None,
        }
    }
}

/// Text styles
pub mod text_style {
    use super::*;

    pub fn primary() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::TEXT_PRIMARY),
        }
    }

    pub fn secondary() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::TEXT_SECONDARY),
        }
    }

    pub fn tertiary() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::TEXT_TERTIARY),
        }
    }

    pub fn on_dark() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::TEXT_ON_DARK),
        }
    }

    pub fn accent() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::ACCENT),
        }
    }

    pub fn brand() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::PRIMARY),
        }
    }

    pub fn success() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::SUCCESS),
        }
    }

    pub fn warning() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::WARNING),
        }
    }

    pub fn error() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::ERROR),
        }
    }

    pub fn send() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::SEND),
        }
    }

    pub fn receive() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::RECEIVE),
        }
    }

    pub fn shielded() -> impl Fn(&Theme) -> text::Style {
        |_| text::Style {
            color: Some(colors::SHIELDED),
        }
    }
}

/// Format ZEC amount with proper precision
pub fn format_zec(zatoshi: u64) -> String {
    let zec = zatoshi as f64 / 100_000_000.0;
    if zec == 0.0 {
        "0.00".to_string()
    } else if zec < 0.0001 {
        format!("{zec:.8}")
    } else if zec < 1.0 {
        format!("{zec:.4}")
    } else {
        format!("{zec:.2}")
    }
}

/// Format ZEC amount for display (larger text)
pub fn format_zec_display(zatoshi: u64) -> (String, String) {
    let zec = zatoshi as f64 / 100_000_000.0;
    let formatted = format!("{zec:.8}");
    let parts: Vec<&str> = formatted.split('.').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), format!(".{}", parts[1]))
    } else {
        (formatted, String::new())
    }
}

/// Truncate address for display
pub fn truncate_address(address: &str, chars: usize) -> String {
    if address.len() <= chars * 2 + 3 {
        address.to_string()
    } else {
        format!(
            "{}...{}",
            &address[..chars],
            &address[address.len() - chars..]
        )
    }
}
