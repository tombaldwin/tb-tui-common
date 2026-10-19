//! Shared drawing vocabulary: rounded panels, decorated titles, pills,
//! separators, `key: value` fields and the width arithmetic that keeps
//! a row whole on a narrow terminal. Lifted from ebman's `ui/chrome.rs`
//! so ebman and pgman draw the same frame.
//!
//! Each app keeps its own `Theme`; it hands these helpers a [`Chrome`]
//! — the icon style and the handful of colours they need — built from
//! it. Every function is pure.

use crate::theme::IconStyle;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders};

/// What the chrome helpers need from an app's theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chrome {
    pub icons: IconStyle,
    /// Border of a panel that doesn't have focus.
    pub border_idle: Color,
    /// Border of the panel that has focus.
    pub border_active: Color,
    /// Labels, separators, secondary text.
    pub muted: Color,
    /// Values.
    pub text: Color,
}

pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
pub const ASCII_SPINNER: &[&str] = &["|", "/", "-", "\\"];

/// A spinner frame for `elapsed_ms`, ten a second.
pub fn spinner(elapsed_ms: u128, icons: IconStyle) -> &'static str {
    let frames = match icons {
        IconStyle::Ascii => ASCII_SPINNER,
        IconStyle::Unicode | IconStyle::Powerline => SPINNER_FRAMES,
    };
    frames[(elapsed_ms / 100) as usize % frames.len()]
}

/// The ascii fallback for a decorative glyph under `icons = "ascii"`.
pub fn glyph<'a>(icons: IconStyle, unicode: &'a str, ascii: &'a str) -> &'a str {
    match icons {
        IconStyle::Ascii => ascii,
        _ => unicode,
    }
}

/// A bordered panel with rounded corners.
pub fn rounded_block(c: &Chrome, active: bool) -> Block<'static> {
    let colour = if active {
        c.border_active
    } else {
        c.border_idle
    };
    Block::default()
        .borders(Borders::ALL)
        .border_type(match c.icons {
            // Rounded corners are unicode; ascii keeps the square set,
            // which every terminal font has.
            IconStyle::Ascii => BorderType::Plain,
            _ => BorderType::Rounded,
        })
        .border_style(Style::default().fg(colour))
}

/// A panel title as it sits in the border: `[ ◆ name ◆ ]`, `[ name ]`
/// in ascii, and ` name ` in Powerline — no caps. The rounded caps
/// (U+E0B6 / U+E0B4) were tried and dropped: ebman never rendered them,
/// and in pgman they read as clutter on every panel.
pub fn decorate_title(icons: IconStyle, title: &str) -> String {
    let t = title.trim();
    match icons {
        IconStyle::Ascii => format!("[ {t} ]"),
        IconStyle::Powerline => format!(" {t} "),
        IconStyle::Unicode => format!("[ ◆ {t} ◆ ]"),
    }
}

/// [`rounded_block`] with a decorated, bold title in `accent`.
pub fn titled_block(c: &Chrome, title: &str, active: bool, accent: Color) -> Block<'static> {
    rounded_block(c, active).title(Span::styled(
        decorate_title(c.icons, title),
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    ))
}

/// A bold label on a coloured background, padded one cell each side.
pub fn pill(text: &str, fg: Color, bg: Color) -> Span<'static> {
    Span::styled(
        format!(" {text} "),
        Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
    )
}

/// Pills in a row: joined by [`sep`], or in Powerline by triangles whose
/// colours flow from one pill into the next.
pub fn pill_chain(items: &[(String, Color, Color)], c: &Chrome) -> Vec<Span<'static>> {
    let mut spans = Vec::with_capacity(items.len() * 2 + 1);
    if items.is_empty() {
        return spans;
    }
    if c.icons == IconStyle::Powerline {
        spans.push(Span::styled("\u{e0b2}", Style::default().fg(items[0].2)));
        for (i, (text, fg, bg)) in items.iter().enumerate() {
            spans.push(pill(text, *fg, *bg));
            let bridge = match items.get(i + 1) {
                Some(next) => Style::default().fg(*bg).bg(next.2),
                None => Style::default().fg(*bg),
            };
            spans.push(Span::styled("\u{e0b0}", bridge));
        }
    } else {
        for (i, (text, fg, bg)) in items.iter().enumerate() {
            if i > 0 {
                spans.push(sep(c));
            }
            spans.push(pill(text, *fg, *bg));
        }
    }
    spans
}

/// [`sep`]'s width in every icon style.
pub const SEP_WIDTH: usize = 5;

/// The separator between header fields.
pub fn sep(c: &Chrome) -> Span<'static> {
    // U+E0B1, the thin powerline divider, needs a patched font.
    let s = match c.icons {
        IconStyle::Powerline => "  \u{e0b1}  ",
        IconStyle::Ascii => "  |  ",
        IconStyle::Unicode => "  •  ",
    };
    Span::styled(s, Style::default().fg(c.muted))
}

/// `Key: value`, the key muted, the value bold.
pub fn kv(key: &str, value: &str, c: &Chrome) -> Vec<Span<'static>> {
    vec![
        Span::styled(format!("{key}: "), Style::default().fg(c.muted)),
        Span::styled(
            value.to_string(),
            Style::default().fg(c.text).add_modifier(Modifier::BOLD),
        ),
    ]
}

/// How many fields of `field_widths` fit in `available` columns with a
/// separator between each. At least one: a row showing nothing is worse
/// than one showing a clipped first field.
pub fn fields_that_fit(field_widths: &[usize], sep_width: usize, available: usize) -> usize {
    if field_widths.is_empty() {
        return 0;
    }
    let mut used = 0usize;
    let mut fitted = 0usize;
    for (i, w) in field_widths.iter().enumerate() {
        let extra = if i == 0 { *w } else { sep_width + *w };
        if used + extra > available {
            break;
        }
        used += extra;
        fitted += 1;
    }
    fitted.max(1)
}

/// Join field groups with [`sep`], dropping whole groups that don't fit
/// `available` columns. Never part of one: ratatui clips the right edge,
/// which renders a label promising a value it then cut in half.
pub fn join_fields_to_fit<'a>(
    groups: Vec<Vec<Span<'a>>>,
    c: &Chrome,
    available: u16,
) -> Vec<Span<'a>> {
    let widths: Vec<usize> = groups
        .iter()
        .map(|g| g.iter().map(Span::width).sum())
        .collect();
    let keep = fields_that_fit(&widths, SEP_WIDTH, available as usize);
    let mut out = Vec::new();
    for (i, g) in groups.into_iter().take(keep).enumerate() {
        if i > 0 {
            out.push(sep(c));
        }
        out.extend(g);
    }
    out
}

/// Trim a key-hint strip (hints separated by two spaces) to whole hints,
/// so a narrow terminal never shows a key whose action was cut off. The
/// first hint always stays, cut only if it alone is wider than the line.
pub fn hints_to_fit(line: &str, available: u16) -> String {
    let available = available as usize;
    let width = |s: &str| Span::raw(s).width();
    if width(line) <= available {
        return line.to_string();
    }
    let mut out = String::new();
    for hint in line.trim_start().split("  ").filter(|h| !h.is_empty()) {
        if !out.is_empty() && width(&out) + 2 + width(hint) > available {
            break;
        }
        out.push_str(if out.is_empty() { " " } else { "  " });
        out.push_str(hint);
    }
    if width(&out) > available {
        out = out.chars().take(available).collect();
    }
    out
}

/// Marker in front of the selected row.
pub fn cursor_marker(icons: IconStyle) -> &'static str {
    match icons {
        IconStyle::Powerline => "\u{e0b0} ",
        IconStyle::Ascii => "> ",
        IconStyle::Unicode => "▌ ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chrome(icons: IconStyle) -> Chrome {
        Chrome {
            icons,
            border_idle: Color::Gray,
            border_active: Color::Cyan,
            muted: Color::DarkGray,
            text: Color::White,
        }
    }

    #[test]
    fn titles_are_decorated_per_icon_style() {
        assert_eq!(
            decorate_title(IconStyle::Unicode, " editor "),
            "[ ◆ editor ◆ ]"
        );
        assert_eq!(decorate_title(IconStyle::Ascii, "editor"), "[ editor ]");
        assert_eq!(decorate_title(IconStyle::Powerline, "editor"), " editor ");
    }

    #[test]
    fn the_separator_is_as_wide_as_it_says_in_every_style() {
        for icons in [IconStyle::Unicode, IconStyle::Ascii, IconStyle::Powerline] {
            assert_eq!(sep(&chrome(icons)).width(), SEP_WIDTH, "{icons:?}");
        }
        assert!(sep(&chrome(IconStyle::Ascii)).content.is_ascii());
    }

    #[test]
    fn whole_fields_are_dropped_never_half_of_one() {
        assert_eq!(fields_that_fit(&[10, 10, 10], 5, 25), 2);
        assert_eq!(fields_that_fit(&[10, 10, 10], 5, 24), 1);
        assert_eq!(fields_that_fit(&[40], 5, 10), 1, "always one");
        assert_eq!(fields_that_fit(&[], 5, 10), 0);
        let c = chrome(IconStyle::Unicode);
        let joined = join_fields_to_fit(
            vec![kv("Server", "db:5432", &c), kv("Database", "shop", &c)],
            &c,
            20,
        );
        let text: String = joined.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(
            text, "Server: db:5432",
            "Database didn't fit, so it went whole"
        );
    }

    #[test]
    fn hint_strips_keep_whole_hints() {
        let line = " j/k move  e editor  ? help  q quit";
        assert_eq!(hints_to_fit(line, 80), line);
        assert_eq!(hints_to_fit(line, 20), " j/k move  e editor");
        assert_eq!(
            hints_to_fit(line, 4),
            " j/k",
            "the first hint stays, cut to fit"
        );
    }

    #[test]
    fn pill_chains_flow_in_powerline_and_take_seps_otherwise() {
        let items = vec![
            ("RO".to_string(), Color::Black, Color::Green),
            ("TAP".to_string(), Color::Black, Color::Blue),
        ];
        let plain = pill_chain(&items, &chrome(IconStyle::Unicode));
        let text: String = plain.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(
            text, " RO   •   TAP ",
            "joined by sep, as header fields are"
        );
        let pl = pill_chain(&items, &chrome(IconStyle::Powerline));
        assert_eq!(pl.len(), 5, "lead-in, pill, bridge, pill, tail");
        assert_eq!(
            pl[2].style.bg,
            Some(Color::Blue),
            "the bridge flows into the next pill"
        );
        assert!(pill_chain(&[], &chrome(IconStyle::Unicode)).is_empty());
    }

    #[test]
    fn ascii_mode_stays_ascii() {
        let c = chrome(IconStyle::Ascii);
        assert!(cursor_marker(c.icons).is_ascii());
        assert!(spinner(250, c.icons).is_ascii());
        assert_eq!(glyph(c.icons, "◆", "*"), "*");
        assert!(decorate_title(c.icons, "x").is_ascii());
    }
}
