use crate::fs;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[cfg(feature = "wgpu")]
use iced_wgpu::Renderer as IRenderer;

#[cfg(feature = "opengl")]
use iced_glow::Renderer as IRenderer;

// TODO 
// finish carryover from original stylesheet here: https://github.com/ajour/ajour/blob/master/src/gui/style.rs
pub mod application;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod container;
pub mod modal;
pub mod header;
pub mod picklist;
pub mod scrollable;
pub mod text;
pub mod text_input;
pub mod table_row;

pub use button::ButtonStyle;
pub use card::CardStyle;
pub use checkbox::CheckboxStyle;
pub use container::ContainerStyle;
pub use modal::ModalStyle;
pub use picklist::PickListStyle;
pub use scrollable::ScrollableStyle;
pub use text_input::TextInputStyle;
pub use table_row::TableRowStyle;
pub use header::HeaderStyle;

pub async fn load_user_themes() -> Vec<Theme> {
    log::debug!("loading user themes");

    fs::load_user_themes().await
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Theme {
    pub name: String,
    pub palette: ColorPalette,
}

pub type Renderer = IRenderer<Theme>;
pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
pub type Container<'a, Message> = iced::widget::Container<'a, Message, Renderer>;
pub type Column<'a, Message> = iced::widget::Column<'a, Message, Renderer>;
pub type Row<'a, Message> = iced::widget::Row<'a, Message, Renderer>;
pub type Text<'a> = iced::widget::Text<'a, Renderer>;
pub type TextInput<'a, Message> = iced::widget::TextInput<'a, Message, Renderer>;
pub type Button<'a, Message> = iced::widget::Button<'a, Message, Renderer>;
pub type Scrollable<'a, Message> = iced::widget::Scrollable<'a, Message, Renderer>;
pub type PickList<'a, T, Message> = iced::widget::PickList<'a, T, Message, Renderer>;
pub type Card<'a, Message> = iced_aw::native::Card<'a, Message, Renderer>;
pub type Modal<'a, Content, Message> = iced_aw::modal::Modal<'a, Content, Message, Renderer, Theme>;
pub type Header<'a, Message> = grin_gui_widgets::widget::header::Header<'a, Message, Renderer>;
pub type TableRow<'a, Message> = grin_gui_widgets::widget::table_row::TableRow<'a, Message, Renderer>;


#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct BaseColors {
    #[serde(with = "serde_color")]
    pub background: iced_native::Color,
    #[serde(with = "serde_color")]
    pub foreground: iced_native::Color,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct NormalColors {
    #[serde(with = "serde_color")]
    pub primary: iced_native::Color,
    #[serde(with = "serde_color")]
    pub secondary: iced_native::Color,
    #[serde(with = "serde_color")]
    pub surface: iced_native::Color,
    #[serde(with = "serde_color")]
    pub error: iced_native::Color,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct BrightColors {
    #[serde(with = "serde_color")]
    pub primary: iced_native::Color,
    #[serde(with = "serde_color")]
    pub secondary: iced_native::Color,
    #[serde(with = "serde_color")]
    pub surface: iced_native::Color,
    #[serde(with = "serde_color")]
    pub error: iced_native::Color,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct ColorPalette {
    pub base: BaseColors,
    pub normal: NormalColors,
    pub bright: BrightColors,
}

impl Theme {
    pub fn all() -> Vec<(String, Theme)> {
        vec![
            ("Alliance".to_string(), Theme::alliance()),
            ("Ayu".to_string(), Theme::ayu()),
            ("Dark".to_string(), Theme::dark()),
            ("Dracula".to_string(), Theme::dracula()),
            ("Ferra".to_string(), Theme::ferra()),
            ("Forest Night".to_string(), Theme::forest_night()),
            ("Gruvbox".to_string(), Theme::gruvbox()),
            ("Horde".to_string(), Theme::horde()),
            ("Light".to_string(), Theme::light()),
            ("Nord".to_string(), Theme::nord()),
            ("One Dark".to_string(), Theme::one_dark()),
            ("Outrun".to_string(), Theme::outrun()),
            ("Solarized Dark".to_string(), Theme::solarized_dark()),
            ("Solarized Light".to_string(), Theme::solarized_light()),
            ("Sort".to_string(), Theme::sort()),
        ]
    }

    pub fn dark() -> Theme {
        Theme {
            name: "Dark".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#111111").unwrap(),
                    foreground: hex_to_color("#1C1C1C").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#3f2b56").unwrap(),
                    secondary: hex_to_color("#4a3c1c").unwrap(),
                    surface: hex_to_color("#828282").unwrap(),
                    error: hex_to_color("#992B2B").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#BA84FC").unwrap(),
                    secondary: hex_to_color("#ffd03c").unwrap(),
                    surface: hex_to_color("#E0E0E0").unwrap(),
                    error: hex_to_color("#C13047").unwrap(),
                },
            },
        }
    }

    pub fn light() -> Theme {
        Theme {
            name: "Light".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#ffffff").unwrap(),
                    foreground: hex_to_color("#F5F5F5").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#DFDBFF").unwrap(),
                    secondary: hex_to_color("#F9D659").unwrap(),
                    surface: hex_to_color("#828282").unwrap(),
                    error: hex_to_color("#992B2B").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#9580ff").unwrap(),
                    secondary: hex_to_color("#EAA326").unwrap(),
                    surface: hex_to_color("#000000").unwrap(),
                    error: hex_to_color("#C13047").unwrap(),
                },
            },
        }
    }

    pub fn alliance() -> Theme {
        Theme {
            name: "Alliance".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#011930").unwrap(),
                    foreground: hex_to_color("#022242").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#3e3523").unwrap(),
                    secondary: hex_to_color("#3e3523").unwrap(),
                    surface: hex_to_color("#7F8387").unwrap(),
                    error: hex_to_color("#5b3a5e").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#ac8a1b").unwrap(),
                    secondary: hex_to_color("#ac8a1b").unwrap(),
                    surface: hex_to_color("#B4B9BF").unwrap(),
                    error: hex_to_color("#953e43").unwrap(),
                },
            },
        }
    }

    pub fn horde() -> Theme {
        Theme {
            name: "Horde".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#161313").unwrap(),
                    foreground: hex_to_color("#211C1C").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#331E1F").unwrap(),
                    secondary: hex_to_color("#542A18").unwrap(),
                    surface: hex_to_color("#5E5B5A").unwrap(),
                    error: hex_to_color("#44282a").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#953e43").unwrap(),
                    secondary: hex_to_color("#e27342").unwrap(),
                    surface: hex_to_color("#9B9897").unwrap(),
                    error: hex_to_color("#953e43").unwrap(),
                },
            },
        }
    }

    pub fn ayu() -> Theme {
        Theme {
            name: "Ayu".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#1f2430").unwrap(),
                    foreground: hex_to_color("#232834").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#987a47").unwrap(),
                    secondary: hex_to_color("#315e6b").unwrap(),
                    surface: hex_to_color("#60697a").unwrap(),
                    error: hex_to_color("#712a34").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#ffcc66").unwrap(),
                    secondary: hex_to_color("#5ccfe6").unwrap(),
                    surface: hex_to_color("#cbccc6").unwrap(),
                    error: hex_to_color("#ff3333").unwrap(),
                },
            },
        }
    }

    pub fn dracula() -> Theme {
        Theme {
            name: "Dracula".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#282a36").unwrap(),
                    foreground: hex_to_color("#353746").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#483e61").unwrap(),
                    secondary: hex_to_color("#386e50").unwrap(),
                    surface: hex_to_color("#a2a4a3").unwrap(),
                    error: hex_to_color("#A13034").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#bd94f9").unwrap(),
                    secondary: hex_to_color("#49eb7a").unwrap(),
                    surface: hex_to_color("#f4f8f3").unwrap(),
                    error: hex_to_color("#ff7ac6").unwrap(),
                },
            },
        }
    }

    pub fn forest_night() -> Theme {
        Theme {
            name: "Forest Night".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#323d43").unwrap(),
                    foreground: hex_to_color("#3c474d").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#505a60").unwrap(),
                    secondary: hex_to_color("#465258").unwrap(),
                    surface: hex_to_color("#999f93").unwrap(),
                    error: hex_to_color("#74484c").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#a7c080").unwrap(),
                    secondary: hex_to_color("#83b6af").unwrap(),
                    surface: hex_to_color("#d8caac").unwrap(),
                    error: hex_to_color("#e68183").unwrap(),
                },
            },
        }
    }

    pub fn gruvbox() -> Theme {
        Theme {
            name: "Gruvbox".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#282828").unwrap(),
                    foreground: hex_to_color("#3c3836").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#63612f").unwrap(),
                    secondary: hex_to_color("#695133").unwrap(),
                    surface: hex_to_color("#928374").unwrap(),
                    error: hex_to_color("#81302e").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#98971a").unwrap(),
                    secondary: hex_to_color("#d79921").unwrap(),
                    surface: hex_to_color("#ebdbb2").unwrap(),
                    error: hex_to_color("#cc241d").unwrap(),
                },
            },
        }
    }

    pub fn nord() -> Theme {
        Theme {
            name: "Nord".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#2e3440").unwrap(),
                    foreground: hex_to_color("#3b4252").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#485b60").unwrap(),
                    secondary: hex_to_color("#425066").unwrap(),
                    surface: hex_to_color("#9196a1").unwrap(),
                    error: hex_to_color("#894f5a").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#8fbcbb").unwrap(),
                    secondary: hex_to_color("#5e81ac").unwrap(),
                    surface: hex_to_color("#eceff4").unwrap(),
                    error: hex_to_color("#bf616a").unwrap(),
                },
            },
        }
    }

    pub fn outrun() -> Theme {
        Theme {
            name: "Outrun".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#0d0821").unwrap(),
                    foreground: hex_to_color("#110A2B").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#330442").unwrap(),
                    secondary: hex_to_color("#6e3e2e").unwrap(),
                    surface: hex_to_color("#484e81").unwrap(),
                    error: hex_to_color("#671a30").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#ff00ff").unwrap(),
                    secondary: hex_to_color("#ff963a").unwrap(),
                    surface: hex_to_color("#757dc8").unwrap(),
                    error: hex_to_color("#db2c3e").unwrap(),
                },
            },
        }
    }

    pub fn solarized_dark() -> Theme {
        Theme {
            name: "Solarized Dark".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#012b36").unwrap(),
                    foreground: hex_to_color("#093642").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#1A615B").unwrap(),
                    secondary: hex_to_color("#523F09").unwrap(),
                    surface: hex_to_color("#63797e").unwrap(),
                    error: hex_to_color("#b80f15").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#2aa096").unwrap(),
                    secondary: hex_to_color("#a37f12").unwrap(),
                    surface: hex_to_color("#93a1a1").unwrap(),
                    error: hex_to_color("#EE2F36").unwrap(),
                },
            },
        }
    }

    pub fn solarized_light() -> Theme {
        Theme {
            name: "Solarized Light".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#fdf6e3").unwrap(),
                    foreground: hex_to_color("#eee8d5").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#BCCCC3").unwrap(),
                    secondary: hex_to_color("#ccbd9e").unwrap(),
                    surface: hex_to_color("#95a3a2").unwrap(),
                    error: hex_to_color("#b80f15").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#2aa096").unwrap(),
                    secondary: hex_to_color("#a37f12").unwrap(),
                    surface: hex_to_color("#4C5D63").unwrap(),
                    error: hex_to_color("#EE2F36").unwrap(),
                },
            },
        }
    }

    pub fn sort() -> Theme {
        Theme {
            name: "Sort".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#000000").unwrap(),
                    foreground: hex_to_color("#101010").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#525252").unwrap(),
                    secondary: hex_to_color("#525252").unwrap(),
                    surface: hex_to_color("#525252").unwrap(),
                    error: hex_to_color("#525252").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#A3A3A3").unwrap(),
                    secondary: hex_to_color("#A3A3A3").unwrap(),
                    surface: hex_to_color("#A3A3A3").unwrap(),
                    error: hex_to_color("#A3A3A3").unwrap(),
                },
            },
        }
    }

    pub fn ferra() -> Theme {
        Theme {
            name: "Ferra".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#211f22").unwrap(),
                    foreground: hex_to_color("#2b292d").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#664A50").unwrap(),
                    secondary: hex_to_color("#855859").unwrap(),
                    surface: hex_to_color("#816961").unwrap(),
                    error: hex_to_color("#713f47").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#b4838d").unwrap(),
                    secondary: hex_to_color("#e5989b").unwrap(),
                    surface: hex_to_color("#fecdb2").unwrap(),
                    error: hex_to_color("#e06b75").unwrap(),
                },
            },
        }
    }

    pub fn one_dark() -> Theme {
        Theme {
            name: "One Dark".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: hex_to_color("#282c34").unwrap(),
                    foreground: hex_to_color("#2c323c").unwrap(),
                },
                normal: NormalColors {
                    primary: hex_to_color("#385c7c").unwrap(),
                    secondary: hex_to_color("#654473").unwrap(),
                    surface: hex_to_color("#5b626e").unwrap(),
                    error: hex_to_color("#713f47").unwrap(),
                },
                bright: BrightColors {
                    primary: hex_to_color("#61afef").unwrap(),
                    secondary: hex_to_color("#c679dd").unwrap(),
                    surface: hex_to_color("#a6adba").unwrap(),
                    error: hex_to_color("#e06b75").unwrap(),
                },
            },
        }
    }
}

fn hex_to_color(hex: &str) -> Option<iced_native::Color> {
    if hex.len() == 7 {
        let hash = &hex[0..1];
        let r = u8::from_str_radix(&hex[1..3], 16);
        let g = u8::from_str_radix(&hex[3..5], 16);
        let b = u8::from_str_radix(&hex[5..7], 16);

        return match (hash, r, g, b) {
            ("#", Ok(r), Ok(g), Ok(b)) => Some(iced_native::Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }),
            _ => None,
        };
    }

    None
}

fn color_to_hex(color: &iced_native::Color) -> String {
    let mut color_str = String::from("#");

    let iced_native::Color { r, g, b, .. } = color;
    color_str.push_str(&format!("{:02X}", (r * 255.0) as u8));
    color_str.push_str(&format!("{:02X}", (g * 255.0) as u8));
    color_str.push_str(&format!("{:02X}", (b * 255.0) as u8));

    color_str
}

impl PartialEq for Theme {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Theme {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Eq for Theme {}

impl Ord for Theme {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

// Newtype on iced::Color so we can impl Deserialzer for it
struct Color(iced_native::Color);

mod serde_color {
    use super::{color_to_hex, hex_to_color, Color};
    use serde::de::{self, Error, Unexpected, Visitor};
    use serde::ser;
    use std::fmt;

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<iced_native::Color, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hex string in the format of '#09ACDF'")
            }

            #[allow(clippy::unnecessary_unwrap)]
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if let Some(color) = hex_to_color(s) {
                    return Ok(Color(color));
                }

                Err(de::Error::invalid_value(Unexpected::Str(s), &self))
            }
        }

        deserializer.deserialize_any(ColorVisitor).map(|c| c.0)
    }

    pub(crate) fn serialize<S>(color: &iced_native::Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&color_to_hex(color))
    }
}

#[cfg(test)]
mod tests {
    use super::{serde_color::deserialize, Theme};
    use serde::de::value::{Error, StrDeserializer};
    use serde::de::IntoDeserializer;

    #[test]
    fn test_hex_color_deser() {
        let colors = [
            "AABBCC", "AABBCG", "#AABBCG", "#AABB091", "#AABBCC", "#AABB09",
        ];

        for (idx, color_str) in colors.iter().enumerate() {
            let deserializer: StrDeserializer<Error> = color_str.into_deserializer();

            let color = deserialize(deserializer);

            if idx < 4 {
                assert!(color.is_err());
            } else {
                assert!(color.is_ok());
            }
        }
    }

    #[test]
    fn test_hex_color_ser() {
        let color = super::NormalColors {
            primary: iced_native::Color::from_rgb(1.0, 1.0, 1.0),
            secondary: iced_native::Color::from_rgb(0.5, 0.6, 0.75789),
            surface: iced_native::Color::from_rgb(0.1, 0.2, 0.3),
            error: iced_native::Color::from_rgb(0.0, 0.0, 0.0),
        };

        let ser = serde_yaml::to_string(&color).unwrap();

        dbg!(&ser);
    }

    #[test]
    fn test_theme_yml_deser() {
        let theme_str = "---
        name: Test
        palette:
          base:
            background: '#484793'
            foreground: '#484793'
          normal:
            primary: '#484793'
            secondary: '#484793'
            surface: '#484793'
            error: '#484793'
          bright:
            primary: '#484793'
            secondary: '#484793'
            surface: '#484793'
            error: '#484793'
        ";

        serde_yaml::from_str::<Theme>(theme_str).unwrap();
    }
}
