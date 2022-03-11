use crate::syntax::SyntaxToken;
use chrono::{Local, TimeZone};
use chrono_tz::{OffsetName, Tz};
use num_format::SystemLocale;
use smartcalc::{Session, SmartCalc};
use std::cell::RefCell;

const LANG: &'static str = "en";

pub struct Calculate {
    app: SmartCalc,
    session: RefCell<Session>,
}

impl Default for Calculate {
    fn default() -> Self {
        let timezone = match localzone::get_local_zone() {
            Some(tz) => match tz.parse::<Tz>() {
                Ok(tz) => {
                    let dt = Local::today().naive_local();
                    tz.offset_from_utc_date(&dt).abbreviation().to_string()
                }
                Err(_) => "UTC".to_string(),
            },
            None => "UTC".to_string(),
        };

        let mut app = SmartCalc::default();
        let locale = SystemLocale::default().unwrap();
        app.set_decimal_seperator(locale.decimal().to_string());
        app.set_thousand_separator(locale.separator().to_string());
        app.set_timezone(timezone).unwrap();

        let mut session = Session::new();
        session.set_language(LANG.to_string());
        let session = RefCell::new(session);

        Self { app, session }
    }
}

impl Calculate {
    pub fn execute(&mut self, input: &str) -> Option<(String, Vec<SyntaxToken>)> {
        self.session.borrow_mut().set_text(input.to_string());
        let res = self.app.execute_session(&self.session);
        match &res.lines[0] {
            Some(line) => match &line.result {
                Ok(result) => {
                    let output = result.output.clone();
                    let tokens = line.ui_tokens.iter().map(|token| token.into()).collect();
                    Some((output, tokens))
                }
                _ => None,
            },
            _ => None,
        }
    }
}
