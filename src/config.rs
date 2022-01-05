use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::color::Color;
use crate::command::Command;
use crate::font::{FontFamily, FontStretch, FontStyle, FontWeight};
use crate::hotkey::Hotkey;
use crate::keyboard::Modifiers;
use crate::mouse::MouseButton;
use crate::xkbcommon_sys as xkb;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub ui: UiConfig,
    pub keys: Vec<Hotkey>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            keys: vec![
                Hotkey::new(
                    xkb::XKB_KEY_1,
                    Modifiers::none(),
                    vec![Command::SelectItem(0)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_2,
                    Modifiers::none(),
                    vec![Command::SelectItem(1)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_3,
                    Modifiers::none(),
                    vec![Command::SelectItem(2)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_4,
                    Modifiers::none(),
                    vec![Command::SelectItem(3)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_5,
                    Modifiers::none(),
                    vec![Command::SelectItem(4)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_6,
                    Modifiers::none(),
                    vec![Command::SelectItem(5)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_7,
                    Modifiers::none(),
                    vec![Command::SelectItem(6)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_8,
                    Modifiers::none(),
                    vec![Command::SelectItem(7)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_9,
                    Modifiers::none(),
                    vec![Command::SelectItem(8)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_j,
                    Modifiers::none(),
                    vec![Command::SelectNextItem],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_Down,
                    Modifiers::none(),
                    vec![Command::SelectNextItem],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_n,
                    Modifiers::control(),
                    vec![Command::SelectNextItem],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_k,
                    Modifiers::none(),
                    vec![Command::SelectPreviousItem],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_Down,
                    Modifiers::none(),
                    vec![Command::SelectPreviousItem],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_p,
                    Modifiers::control(),
                    vec![Command::SelectPreviousItem],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_l,
                    Modifiers::control(),
                    vec![Command::ClickMouseButton(MouseButton::Left)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_Return,
                    Modifiers::none(),
                    vec![Command::ClickMouseButton(MouseButton::Left)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_Return,
                    Modifiers::none(),
                    vec![Command::ClickMouseButton(MouseButton::Left)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_h,
                    Modifiers::none(),
                    vec![Command::ClickMouseButton(MouseButton::Right)],
                ),
                Hotkey::new(
                    xkb::XKB_KEY_Return,
                    Modifiers::shift(),
                    vec![Command::ClickMouseButton(MouseButton::Right)],
                ),
                Hotkey::new(xkb::XKB_KEY_q, Modifiers::none(), vec![Command::HideWindow]),
                Hotkey::new(
                    xkb::XKB_KEY_Escape,
                    Modifiers::none(),
                    vec![Command::HideWindow],
                ),
            ],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct UiConfig {
    pub window_name: Cow<'static, str>,
    pub window_class: Cow<'static, str>,
    pub window_padding: f64,
    pub window_width: f64,
    pub item_padding: f64,
    pub item_gap: f64,
    pub icon_size: f64,
    pub item_corner_radius: f64,
    pub font: FontConfig,
    pub color: ColorConfig,
    pub show_index: bool,
}

impl UiConfig {
    pub fn item_height(&self) -> f64 {
        self.icon_size + self.item_padding * 2.0
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            window_name: Cow::Borrowed("KeyTray"),
            window_class: Cow::Borrowed("KeyTray"),
            window_padding: 8.0,
            window_width: 480.0,
            item_padding: 0.0,
            item_gap: 8.0,
            item_corner_radius: 4.0,
            icon_size: 24.0,
            font: FontConfig::default(),
            color: ColorConfig::default(),
            show_index: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct FontConfig {
    pub family: FontFamily,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
    pub size: f64,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: FontFamily::default(),
            weight: FontWeight::default(),
            style: FontStyle::default(),
            stretch: FontStretch::default(),
            size: 12.0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ColorConfig {
    pub window_background: Color,
    pub normal_item_background: Color,
    pub normal_item_foreground: Color,
    pub selected_item_background: Color,
    pub selected_item_foreground: Color,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            window_background: Color::from_rgb(0x21272b),
            normal_item_background: Color::from_rgb(0x363f45),
            normal_item_foreground: Color::from_rgb(0xe8eaeb),
            selected_item_background: Color::from_rgb(0x1c95e6),
            selected_item_foreground: Color::from_rgb(0xe8eaeb),
        }
    }
}
