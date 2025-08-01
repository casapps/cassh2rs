#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    Dracula,
}

impl Theme {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "dracula" => Theme::Dracula,
            _ => Theme::Dracula, // Default
        }
    }
    
    pub fn primary_color(&self) -> &str {
        match self {
            Theme::Light => "#0066cc",
            Theme::Dark => "#66b3ff",
            Theme::Dracula => "#bd93f9",
        }
    }
    
    pub fn background_color(&self) -> &str {
        match self {
            Theme::Light => "#ffffff",
            Theme::Dark => "#1e1e1e",
            Theme::Dracula => "#282a36",
        }
    }
    
    pub fn text_color(&self) -> &str {
        match self {
            Theme::Light => "#333333",
            Theme::Dark => "#d4d4d4",
            Theme::Dracula => "#f8f8f2",
        }
    }
    
    pub fn error_color(&self) -> &str {
        match self {
            Theme::Light => "#cc0000",
            Theme::Dark => "#ff6666",
            Theme::Dracula => "#ff5555",
        }
    }
    
    pub fn success_color(&self) -> &str {
        match self {
            Theme::Light => "#008800",
            Theme::Dark => "#66ff66",
            Theme::Dracula => "#50fa7b",
        }
    }
    
    pub fn warning_color(&self) -> &str {
        match self {
            Theme::Light => "#ff8800",
            Theme::Dark => "#ffcc66",
            Theme::Dracula => "#f1fa8c",
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dracula
    }
}