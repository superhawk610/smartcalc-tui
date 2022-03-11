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
        debug_assert!(
            index <= token.start,
            "expected tokens to be in order and not to overlap"
        );

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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! tkn {
        ($start:literal $end:literal $color:tt) => {
            SyntaxToken {
                start: $start,
                end: $end,
                color: Color::$color,
            }
        };
    }

    macro_rules! check_syntax {
        ($input:literal with $tokens:ident => $expected:expr) => {
            let highlights = syntax_highlight($input, &$tokens);
            let mut output = String::with_capacity(128);
            for h in highlights {
                output.push_str(&format!("{}", h));
            }
            assert_eq!(output, $expected);
        };
    }

    #[test]
    fn test_syntax_highlight() {
        let mut tokens = Vec::new();
        check_syntax!("" with tokens => "");

        // skipping whitespace
        check_syntax!("foo bar" with tokens => "foo bar");
        tokens.push(tkn![0 3 Cyan]);
        check_syntax!("foo bar" with tokens => format!("{} bar", "foo".cyan()));
        tokens.push(tkn![4 7 Red]);
        check_syntax!("foo bar" with tokens => format!("{} {}", "foo".cyan(), "bar".red()));

        // skipping text at the start
        let tokens = vec![tkn![4 7 Red]];
        check_syntax!("foo bar" with tokens => format!("foo {}", "bar".red()));
    }
}
