use crate::{frontend::ast::parse::ASTParser, utils::error::SyntaxError};

use super::{
    base::{FSRPosition, FSRToken},
    expr::FSRExpr, ASTContext,
};

#[derive(PartialEq)]
enum GetterState {
    Name,
    Start,
    _Args,
    _WaitToken,
}

#[derive(Debug, Clone)]
pub struct FSRGetter {
    name: String,
    getter: Box<FSRToken>,
    len: usize,
    pub(crate)single_op: Option<&'static str>,
    meta: FSRPosition,
    pub(crate) is_defined: bool,
}

impl FSRGetter {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn is_unnamed(&self) -> bool {
        self.name.is_empty()
    }

    pub fn get_getter(&self) -> &FSRToken {
        &self.getter
    }

    pub fn parse(source: &[u8], meta: FSRPosition, context: &mut ASTContext) -> Result<Self, SyntaxError> {
        let mut state = GetterState::Start;
        let mut start = 0;
        let mut length = 0;
        let name;
        if source[start] == b'[' {
            name = std::str::from_utf8(&source[start..start + length]).unwrap();
        } else {
            loop {
                let i = source[start];
                let t_i = source[start + length];
                if state == GetterState::Start && ASTParser::is_blank_char_with_new_line(i) {
                    start += 1;
                    continue;
                }

                if ASTParser::is_name_letter(i) && state == GetterState::Start {
                    state = GetterState::Name;
                    continue;
                }

                if state == GetterState::Name && ASTParser::is_name_letter(t_i) {
                    length += 1;
                    continue;
                }

                if state == GetterState::Name && t_i as char == '[' {
                    name = std::str::from_utf8(&source[start..start + length]).unwrap();
                    start += length;
                    start += 1;
                    break;
                }
            }
        }

        let s = std::str::from_utf8(source).unwrap();
        let first = s.find('[').unwrap();
        let last = s.rfind(']').unwrap();
        let args = &source[first + 1..last];
        let sub_meta = meta.from_offset(start);
        let getter = FSRExpr::parse(args, true, sub_meta, context)?;
        Ok(Self {
            name: name.to_string(),
            len: 0,
            single_op: None,
            meta,
            getter: Box::new(getter.0),
            is_defined: false,
        })
    }

    pub fn is_valid_char(c: u8) -> bool {
        let c = c as char;
        c.is_ascii_hexdigit() || c == ':'
    }
}
