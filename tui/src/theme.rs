use ratatui::style::Color;

pub enum ThemeVariant {
    Light,
}

pub struct Theme {
    pub variant: ThemeVariant,
}

impl Theme {
    pub fn new(variant: ThemeVariant) -> Self {
        Self { variant }
    }

    pub fn background(&self) -> Color {
        match self.variant {
            ThemeVariant::Light => Color::Rgb(245, 245, 255),
        }
    }

    pub fn text(&self) -> Color {
        match self.variant {
            ThemeVariant::Light => Color::Rgb(153, 78, 236),
        }
    }

    pub fn border(&self) -> Color {
        match self.variant {
            ThemeVariant::Light => Color::Rgb(255, 87, 51),
        }
    }

    pub fn selected(&self) -> Color {
        match self.variant {
            ThemeVariant::Light => Color::Rgb(151, 251, 151),
        }
    }

    pub fn selectable(&self) -> Color {
        match self.variant {
            ThemeVariant::Light => Color::Rgb(38, 38, 38),
        }
    }

    pub fn header(&self) -> Color {
        match self.variant {
            ThemeVariant::Light => Color::Rgb(0, 150, 255),
        }
    }

    pub fn negative(&self) -> Color {
        match self.variant {
            ThemeVariant::Light => Color::Rgb(255, 51, 51),
        }
    }

    pub fn positive(&self) -> Color {
        match self.variant {
            ThemeVariant::Light => Color::Rgb(51, 51, 255),
        }
    }

    pub fn autocomplete(&self) -> Color {
        match self.variant {
            ThemeVariant::Light => Color::Rgb(128, 128, 128),
        }
    }
}
