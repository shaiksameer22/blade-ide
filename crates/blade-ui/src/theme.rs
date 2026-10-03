use ratatui::style::Color;
// use ratatui::style::Style;
// use blade_syntax::highlighter::HighlightType;

/// Theme colors for the editor
pub struct Theme {
    pub name: String,
    // Editor
    pub editor_bg: Color,
    pub editor_fg: Color,
    pub line_number: Color,
    pub line_number_active: Color,
    pub cursor: Color,
    pub selection: Color,
    pub current_line: Color,
    // UI
    pub sidebar_bg: Color,
    pub sidebar_fg: Color,
    pub statusbar_bg: Color,
    pub statusbar_fg: Color,
    pub tab_active_bg: Color,
    pub tab_active_fg: Color,
    pub tab_inactive_bg: Color,
    pub tab_inactive_fg: Color,
    // Syntax
    pub keyword: Color,
    pub string: Color,
    pub comment: Color,
    pub function: Color,
    pub type_color: Color,
    pub variable: Color,
    pub number: Color,
    pub operator: Color,
    pub constant: Color,
    pub error: Color,
}

impl Theme {
    /// Default dark theme inspired by VS Code Dark+
    pub fn dark() -> Self {
        Self {
            name: "Blade Dark".to_string(),
            editor_bg: Color::Rgb(30, 30, 30),
            editor_fg: Color::Rgb(212, 212, 212),
            line_number: Color::Rgb(110, 110, 110),
            line_number_active: Color::Rgb(200, 200, 200),
            cursor: Color::Rgb(255, 255, 255),
            selection: Color::Rgb(38, 79, 120),
            current_line: Color::Rgb(40, 40, 40),
            sidebar_bg: Color::Rgb(37, 37, 38),
            sidebar_fg: Color::Rgb(204, 204, 204),
            statusbar_bg: Color::Rgb(0, 122, 204),
            statusbar_fg: Color::White,
            tab_active_bg: Color::Rgb(30, 30, 30),
            tab_active_fg: Color::White,
            tab_inactive_bg: Color::Rgb(45, 45, 45),
            tab_inactive_fg: Color::Rgb(150, 150, 150),
            keyword: Color::Rgb(86, 156, 214),
            string: Color::Rgb(206, 145, 120),
            comment: Color::Rgb(106, 153, 85),
            function: Color::Rgb(220, 220, 170),
            type_color: Color::Rgb(78, 201, 176),
            variable: Color::Rgb(156, 220, 254),
            number: Color::Rgb(181, 206, 168),
            operator: Color::Rgb(212, 212, 212),
            constant: Color::Rgb(100, 150, 224),
            error: Color::Rgb(244, 71, 71),
        }
    }

    pub fn highlight_color(&self, hl: blade_syntax::highlighter::HighlightType) -> Color {
        use blade_syntax::highlighter::HighlightType::*;
        match hl {
            Keyword | Label | Namespace => self.keyword,
            String => self.string,
            Comment => self.comment,
            Function => self.function,
            Type => self.type_color,
            Variable | Property | Tag | Attribute => self.variable,
            Number => self.number,
            Operator | Punctuation => self.operator,
            Constant | Builtin => self.constant,
            Error => self.error,
            None => self.editor_fg,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
