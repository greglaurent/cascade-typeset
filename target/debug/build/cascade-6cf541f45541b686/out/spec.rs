// GENERATED from tokens.ron by build.rs — do not edit by hand.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    Serif,
    Sans,
    Mono,
}

impl Category {
    pub const ALL: [Category; 3] = [Category::Serif, Category::Sans, Category::Mono];
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Serif => "serif",
            Self::Sans => "sans",
            Self::Mono => "mono",
        }
    }
    pub fn default_optical_size(self) -> &'static str {
        match self {
            Self::Serif => "11pt",
            Self::Sans => "12pt",
            Self::Mono => "12pt",
        }
    }
    pub fn default_x_height(self) -> f64 {
        match self {
            Self::Serif => 0.49,
            Self::Sans => 0.53,
            Self::Mono => 0.535,
        }
    }
    pub fn default_k_tracking(self) -> f64 {
        match self {
            Self::Serif => 0.022,
            Self::Sans => 0.03,
            Self::Mono => 0.0,
        }
    }
    pub fn default_leading_base(self) -> f64 {
        match self {
            Self::Serif => 1.35,
            Self::Sans => 1.45,
            Self::Mono => 1.5,
        }
    }
    pub fn default_word_space(self) -> f64 {
        match self {
            Self::Serif => 0.28,
            Self::Sans => 0.28,
            Self::Mono => 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Font {
    Inter,
    Jost,
    Lora,
}

impl Font {
    pub const ALL: [Font; 3] = [Font::Inter, Font::Jost, Font::Lora];
    pub fn family(self) -> &'static str {
        match self {
            Self::Inter => "Inter",
            Self::Jost => "Jost",
            Self::Lora => "Lora",
        }
    }
    pub fn category(self) -> Category {
        match self {
            Self::Inter => Category::Sans,
            Self::Jost => Category::Sans,
            Self::Lora => Category::Serif,
        }
    }
    pub fn optical_size(self) -> &'static str {
        match self {
            Self::Inter => "12pt",
            Self::Jost => "12pt",
            Self::Lora => "11pt",
        }
    }
    pub fn x_height(self) -> f64 {
        match self {
            Self::Inter => 0.546,
            Self::Jost => 0.46,
            Self::Lora => 0.5,
        }
    }
    pub fn k_tracking(self) -> f64 {
        match self {
            Self::Inter => 0.03,
            Self::Jost => 0.03,
            Self::Lora => 0.022,
        }
    }
    pub fn leading_base(self) -> f64 {
        match self {
            Self::Inter => 1.45,
            Self::Jost => 1.45,
            Self::Lora => 1.38,
        }
    }
    pub fn word_space(self) -> f64 {
        match self {
            Self::Inter => 0.26,
            Self::Jost => 0.28,
            Self::Lora => 0.28,
        }
    }
    pub fn cap_height(self) -> f64 {
        match self {
            Self::Inter => 0.728,
            Self::Jost => 0.7,
            Self::Lora => 0.7,
        }
    }
    pub fn units_per_em(self) -> u32 {
        match self {
            Self::Inter => 2048,
            Self::Jost => 1000,
            Self::Lora => 1000,
        }
    }
    pub fn sx(self) -> &'static str {
        match self {
            Self::Inter => "sxHeight 1118",
            Self::Jost => "sxHeight 460",
            Self::Lora => "sxHeight 500",
        }
    }
    pub fn asc(self) -> &'static str {
        match self {
            Self::Inter => "0.969",
            Self::Jost => "1.07",
            Self::Lora => "1.006",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            Self::Inter => "-0.241",
            Self::Jost => "-0.375",
            Self::Lora => "-0.274",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScalePreset {
    Classical,
    GoldenRatio,
    GoldenDitonic,
    Tritonic,
    Tetratonic,
    MajorThird,
    MinorThird,
}

impl ScalePreset {
    pub const ALL: [ScalePreset; 7] = [ScalePreset::Classical, ScalePreset::GoldenRatio, ScalePreset::GoldenDitonic, ScalePreset::Tritonic, ScalePreset::Tetratonic, ScalePreset::MajorThird, ScalePreset::MinorThird];
    pub fn id(self) -> &'static str {
        match self {
            Self::Classical => "classical",
            Self::GoldenRatio => "golden-ratio",
            Self::GoldenDitonic => "golden-ditonic",
            Self::Tritonic => "tritonic",
            Self::Tetratonic => "tetratonic",
            Self::MajorThird => "major-third",
            Self::MinorThird => "minor-third",
        }
    }
    pub fn ratio(self) -> f64 {
        match self {
            Self::Classical => 2.0,
            Self::GoldenRatio => 1.618033988749895,
            Self::GoldenDitonic => 1.618033988749895,
            Self::Tritonic => 2.0,
            Self::Tetratonic => 2.0,
            Self::MajorThird => 1.25,
            Self::MinorThird => 1.2,
        }
    }
    pub fn n(self) -> u32 {
        match self {
            Self::Classical => 5,
            Self::GoldenRatio => 1,
            Self::GoldenDitonic => 2,
            Self::Tritonic => 3,
            Self::Tetratonic => 4,
            Self::MajorThird => 1,
            Self::MinorThird => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Fg,
    FgMuted,
    FgSubtle,
    Bg,
    BgSubtle,
    Rule,
    Accent,
    AccentHover,
    AccentSubtle,
    AccentRule,
    AccentVisited,
}

impl Color {
    pub const ALL: [Color; 11] = [Color::Fg, Color::FgMuted, Color::FgSubtle, Color::Bg, Color::BgSubtle, Color::Rule, Color::Accent, Color::AccentHover, Color::AccentSubtle, Color::AccentRule, Color::AccentVisited];
    pub fn id(self) -> &'static str {
        match self {
            Self::Fg => "fg",
            Self::FgMuted => "fg-muted",
            Self::FgSubtle => "fg-subtle",
            Self::Bg => "bg",
            Self::BgSubtle => "bg-subtle",
            Self::Rule => "rule",
            Self::Accent => "accent",
            Self::AccentHover => "accent-hover",
            Self::AccentSubtle => "accent-subtle",
            Self::AccentRule => "accent-rule",
            Self::AccentVisited => "accent-visited",
        }
    }
    pub fn light(self) -> &'static str {
        match self {
            Self::Fg => "#171717",
            Self::FgMuted => "#59544C",
            Self::FgSubtle => "#7A746A",
            Self::Bg => "#F6F2E9",
            Self::BgSubtle => "#EFE9DC",
            Self::Rule => "#C4BDB0",
            Self::Accent => "#7A2E28",
            Self::AccentHover => "#5E211C",
            Self::AccentSubtle => "#F0E2DE",
            Self::AccentRule => "#C9A5A0",
            Self::AccentVisited => "#5A3A52",
        }
    }
    pub fn dark(self) -> &'static str {
        match self {
            Self::Fg => "#E8E4DC",
            Self::FgMuted => "#A8A196",
            Self::FgSubtle => "#8A8378",
            Self::Bg => "#14120E",
            Self::BgSubtle => "#1E1B16",
            Self::Rule => "#3A362F",
            Self::Accent => "#E09A93",
            Self::AccentHover => "#EFB8B1",
            Self::AccentSubtle => "#2E1614",
            Self::AccentRule => "#4A2A26",
            Self::AccentVisited => "#C9A5C4",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Multiplier {
    N1,
    Base,
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
}

impl Multiplier {
    pub const ALL: [Multiplier; 8] = [Multiplier::N1, Multiplier::Base, Multiplier::P1, Multiplier::P2, Multiplier::P3, Multiplier::P4, Multiplier::P5, Multiplier::P6];
    pub fn id(self) -> &'static str {
        match self {
            Self::N1 => "n1",
            Self::Base => "base",
            Self::P1 => "p1",
            Self::P2 => "p2",
            Self::P3 => "p3",
            Self::P4 => "p4",
            Self::P5 => "p5",
            Self::P6 => "p6",
        }
    }
    pub fn factor(self) -> f64 {
        match self {
            Self::N1 => 0.5,
            Self::Base => 1.0,
            Self::P1 => 2.0,
            Self::P2 => 3.0,
            Self::P3 => 4.0,
            Self::P4 => 6.0,
            Self::P5 => 8.0,
            Self::P6 => 12.0,
        }
    }
}

pub const SCALE_DEFAULT: ScalePreset = ScalePreset::GoldenDitonic;
pub const STEPS_MIN: i32 = -5;
pub const STEPS_MAX: i32 = 5;
pub const WORD_SPACE_K: f64 = 0.04;
pub const TRACKING_CLAMP: f64 = 0.04;
pub const LEADING_CLAMP: (f64, f64) = (1.2, 1.5);
pub const MEASURE: u32 = 65;
