use std::path::Path;

use ratatui::style::Color;
use serde::Deserialize;

#[allow(dead_code)]
pub struct Theme {
    pub base00: Color, // background
    pub base01: Color, // lighter bg / status bars
    pub base02: Color, // selection bg
    pub base03: Color, // comments / subtle
    pub base04: Color, // dark fg
    pub base05: Color, // default fg
    pub base06: Color, // light fg
    pub base07: Color, // light bg
    pub base08: Color, // red (errors, disabled)
    pub base09: Color, // peach (integers, constants)
    pub base0a: Color, // yellow (classes, skills)
    pub base0b: Color, // green (strings, enabled)
    pub base0c: Color, // teal (support, status msgs)
    pub base0d: Color, // blue (functions)
    pub base0e: Color, // mauve (keywords)
    pub base0f: Color, // flamingo (deprecated)
}

pub struct NamedTheme {
    pub name: String,
    pub theme: Theme,
}

fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
}

fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    Some(rgb(u32::from_str_radix(s, 16).ok()?))
}

#[derive(Deserialize)]
struct ThemeFile {
    scheme: String,
    base00: String,
    base01: String,
    base02: String,
    base03: String,
    base04: String,
    base05: String,
    base06: String,
    base07: String,
    base08: String,
    base09: String,
    base0a: String,
    base0b: String,
    base0c: String,
    base0d: String,
    base0e: String,
    base0f: String,
}

impl ThemeFile {
    fn into_named_theme(self) -> Option<NamedTheme> {
        Some(NamedTheme {
            name: self.scheme,
            theme: Theme {
                base00: parse_hex_color(&self.base00)?,
                base01: parse_hex_color(&self.base01)?,
                base02: parse_hex_color(&self.base02)?,
                base03: parse_hex_color(&self.base03)?,
                base04: parse_hex_color(&self.base04)?,
                base05: parse_hex_color(&self.base05)?,
                base06: parse_hex_color(&self.base06)?,
                base07: parse_hex_color(&self.base07)?,
                base08: parse_hex_color(&self.base08)?,
                base09: parse_hex_color(&self.base09)?,
                base0a: parse_hex_color(&self.base0a)?,
                base0b: parse_hex_color(&self.base0b)?,
                base0c: parse_hex_color(&self.base0c)?,
                base0d: parse_hex_color(&self.base0d)?,
                base0e: parse_hex_color(&self.base0e)?,
                base0f: parse_hex_color(&self.base0f)?,
            },
        })
    }
}

fn fallback_theme() -> NamedTheme {
    NamedTheme {
        name: "Terminal Default".into(),
        theme: Theme {
            base00: Color::Reset,
            base01: Color::Reset,
            base02: Color::DarkGray,
            base03: Color::DarkGray,
            base04: Color::Gray,
            base05: Color::Reset,
            base06: Color::Reset,
            base07: Color::White,
            base08: Color::Red,
            base09: Color::Yellow,
            base0a: Color::Yellow,
            base0b: Color::Green,
            base0c: Color::Cyan,
            base0d: Color::Blue,
            base0e: Color::Magenta,
            base0f: Color::Red,
        },
    }
}

/// Load all `*.yaml` / `*.yml` theme files from `<base_dir>/themes/`,
/// sorted by filename. Falls back to a built-in terminal-default theme when
/// the directory is missing or empty so the TUI always has at least one theme.
pub fn all_themes(base_dir: &Path) -> Vec<NamedTheme> {
    let themes_dir = base_dir.join("themes");
    let Ok(entries) = std::fs::read_dir(&themes_dir) else {
        return vec![fallback_theme()];
    };
    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    paths.sort();
    let mut themes = Vec::new();
    for path in paths {
        let Ok(content) = std::fs::read_to_string(&path) else {
            eprintln!("warning: could not read theme file {}", path.display());
            continue;
        };
        match serde_yaml_ng::from_str::<ThemeFile>(&content) {
            Ok(tf) => match tf.into_named_theme() {
                Some(nt) => themes.push(nt),
                None => eprintln!("warning: invalid hex colors in {}", path.display()),
            },
            Err(e) => eprintln!("warning: skipping {}: {e}", path.display()),
        }
    }
    if themes.is_empty() {
        return vec![fallback_theme()];
    }
    themes
}
