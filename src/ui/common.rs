use vizia::prelude::*;

/// Parse a hex color string like "#4a90d9" and return a Color with the given alpha (0-255).
pub fn hex_color(hex: &str, alpha: u8) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::rgba(r, g, b, alpha)
}

/// Track display data for the static prototype.
pub struct TrackData {
    pub name: &'static str,
    pub color: &'static str,
    pub icon: &'static str,
    pub muted: bool,
    pub soloed: bool,
    pub armed: bool,
    pub selected: bool,
    pub fader_pct: f32,
    pub level_pct: f32,
    pub regions: &'static [(f32, f32)], // (left_pct, width_pct)
}

pub const TRACKS: &[TrackData] = &[
    TrackData {
        name: "Kick",
        color: "#4a90d9",
        icon: "K",
        muted: false,
        soloed: false,
        armed: false,
        selected: true,
        fader_pct: 0.80,
        level_pct: 0.55,
        regions: &[(5.0, 15.0), (35.0, 20.0)],
    },
    TrackData {
        name: "Snare",
        color: "#4a90d9",
        icon: "S",
        muted: false,
        soloed: false,
        armed: false,
        selected: false,
        fader_pct: 0.70,
        level_pct: 0.45,
        regions: &[(5.0, 10.0), (25.0, 15.0), (50.0, 10.0)],
    },
    TrackData {
        name: "HiHat",
        color: "#4a90d9",
        icon: "H",
        muted: false,
        soloed: true,
        armed: false,
        selected: false,
        fader_pct: 0.50,
        level_pct: 0.30,
        regions: &[(5.0, 45.0)],
    },
    TrackData {
        name: "Bass",
        color: "#6fcf97",
        icon: "B",
        muted: false,
        soloed: false,
        armed: false,
        selected: false,
        fader_pct: 0.75,
        level_pct: 0.50,
        regions: &[(5.0, 50.0)],
    },
    TrackData {
        name: "Synth Pad",
        color: "#bb6bd9",
        icon: "P",
        muted: false,
        soloed: false,
        armed: false,
        selected: false,
        fader_pct: 0.40,
        level_pct: 0.25,
        regions: &[(10.0, 40.0)],
    },
    TrackData {
        name: "Lead",
        color: "#bb6bd9",
        icon: "L",
        muted: false,
        soloed: false,
        armed: true,
        selected: false,
        fader_pct: 0.60,
        level_pct: 0.35,
        regions: &[(20.0, 30.0)],
    },
    TrackData {
        name: "FX",
        color: "#f2994a",
        icon: "F",
        muted: true,
        soloed: false,
        armed: false,
        selected: false,
        fader_pct: 0.30,
        level_pct: 0.15,
        regions: &[(40.0, 10.0)],
    },
    TrackData {
        name: "Master",
        color: "#8b8b8b",
        icon: "M",
        muted: false,
        soloed: false,
        armed: false,
        selected: false,
        fader_pct: 0.90,
        level_pct: 0.60,
        regions: &[],
    },
];
