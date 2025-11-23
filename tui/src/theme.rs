use ratatui::style::Color;
use strum_macros::FromRepr;

#[repr(usize)]
#[derive(FromRepr, Clone, Copy)]
pub enum ThemeVariant {
    Light,
    Dracula,
    CatppuccinLatte,
    CatppuccinFrappe,
}

impl ThemeVariant {
    pub fn next_variant(self) -> Self {
        match self {
            ThemeVariant::Light => ThemeVariant::Dracula,
            ThemeVariant::Dracula => ThemeVariant::CatppuccinLatte,
            ThemeVariant::CatppuccinLatte => ThemeVariant::CatppuccinFrappe,
            ThemeVariant::CatppuccinFrappe => ThemeVariant::Light,
        }
    }
}

pub struct Theme {
    pub variant: ThemeVariant,
    palette: ThemePalette,
}

impl Theme {
    pub fn new(variant: ThemeVariant) -> Self {
        let palette = ThemePalette::from_variant(variant);
        Self { variant, palette }
    }

    pub fn new_index(index: usize) -> Self {
        let variant = ThemeVariant::from_repr(index).unwrap_or(ThemeVariant::Light);
        Self::new(variant)
    }

    pub fn next(&mut self) -> usize {
        self.variant = self.variant.next_variant();
        self.palette = ThemePalette::from_variant(self.variant);

        self.variant as usize
    }

    pub fn background(&self) -> Color {
        self.palette.background
    }

    pub fn text(&self) -> Color {
        self.palette.text
    }

    pub fn border(&self) -> Color {
        self.palette.border
    }

    pub fn selected(&self) -> Color {
        self.palette.selected
    }

    pub fn selectable(&self) -> Color {
        self.palette.selectable
    }

    pub fn header(&self) -> Color {
        self.palette.header
    }

    pub fn negative(&self) -> Color {
        self.palette.negative
    }

    pub fn positive(&self) -> Color {
        self.palette.positive
    }

    pub fn autocomplete(&self) -> Color {
        self.palette.autocomplete
    }

    pub fn add_reverse_modifier(&self) -> bool {
        match self.variant {
            ThemeVariant::Light => false,
            ThemeVariant::Dracula => false,
            ThemeVariant::CatppuccinLatte => true,
            ThemeVariant::CatppuccinFrappe => true,
        }
    }
}

struct ThemePalette {
    background: Color,
    text: Color,
    border: Color,
    selected: Color,
    selectable: Color,
    header: Color,
    negative: Color,
    positive: Color,
    autocomplete: Color,
}

impl ThemePalette {
    fn from_variant(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Light => Self {
                background: Color::Rgb(245, 245, 255),
                text: Color::Rgb(153, 78, 236),
                border: Color::Rgb(255, 87, 51),
                selected: Color::Rgb(151, 251, 151),
                selectable: Color::Rgb(38, 38, 38),
                header: Color::Rgb(0, 150, 255),
                negative: Color::Rgb(255, 51, 51),
                positive: Color::Rgb(51, 51, 255),
                autocomplete: Color::Rgb(128, 128, 128),
            },

            ThemeVariant::Dracula => Self {
                background: Color::Rgb(40, 42, 54),
                text: Color::Rgb(248, 248, 242),
                border: Color::Rgb(189, 147, 249),
                selected: Color::Rgb(98, 114, 164),
                selectable: Color::Rgb(255, 85, 85),
                header: Color::Rgb(80, 250, 123),
                negative: Color::Rgb(255, 85, 85),
                positive: Color::Rgb(80, 250, 123),
                autocomplete: Color::Rgb(139, 233, 253),
            },

            ThemeVariant::CatppuccinLatte => Self {
                background: Color::Rgb(239, 241, 245),
                text: Color::Rgb(76, 79, 105),
                border: Color::Rgb(30, 102, 245),
                selected: Color::Rgb(254, 100, 11),
                selectable: Color::Rgb(204, 208, 218),
                header: Color::Rgb(114, 135, 253),
                negative: Color::Rgb(210, 15, 57),
                positive: Color::Rgb(64, 160, 43),
                autocomplete: Color::Rgb(108, 111, 133),
            },

            ThemeVariant::CatppuccinFrappe => Self {
                background: Color::Rgb(48, 52, 70),
                text: Color::Rgb(198, 208, 245),
                border: Color::Rgb(202, 158, 230),
                selected: Color::Rgb(239, 159, 118),
                selectable: Color::Rgb(98, 104, 128),
                header: Color::Rgb(140, 170, 238),
                negative: Color::Rgb(231, 130, 132),
                positive: Color::Rgb(166, 218, 149),
                autocomplete: Color::Rgb(127, 132, 156),
            },
        }
    }
}
