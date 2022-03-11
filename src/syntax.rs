use colored::{Color, ColoredString, Colorize};

pub struct SyntaxToken {
    pub start: usize,
    pub end: usize,
    pub color: Color,
}

impl From<&smartcalc::UiToken> for SyntaxToken {
    fn from(token: &smartcalc::UiToken) -> Self {
        Self {
            start: token.start,
            end: token.end,
            color: syntax_color(&token.ui_type),
        }
    }
}

fn syntax_color(tt: &smartcalc::UiTokenType) -> Color {
    use smartcalc::UiTokenType::*;
    match tt {
        Text => Color::White,
        VariableDefination => Color::White,
        VariableUse => Color::White,
        Comment => Color::White,
        Month => Color::White,
        DateTime => Color::White,
        Number => Color::Cyan,
        Symbol1 => Color::Red,
        Symbol2 => Color::Red,
        Operator => Color::Yellow,
    }
}

pub fn syntax_highlight(s: &str, tokens: &[SyntaxToken]) -> Vec<ColoredString> {
    let mut index = 0;
    let mut strs = Vec::new();

    for token in tokens.iter() {
        // there's some input not part of a token
        if index < token.start {
            strs.push((&s[index..token.start]).into());
        }

        strs.push((&s[token.start..token.end]).color(token.color));
        index = token.end;
    }

    // check for trailing input that's not part of a token
    if index < s.len() {
        strs.push((&s[index..]).into());
    }

    strs
}
