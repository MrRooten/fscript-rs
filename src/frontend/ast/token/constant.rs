use crate::{
    frontend::ast::token::{base::FSRToken, expr::FSRExpr, for_statement::FSRFor, ASTContext},
    utils::error::SyntaxError,
};

use super::{
    base::{FSRPosition, FSRType},
    expr::SingleOp,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum FSRConstantType {
    String(Vec<u8>),
    Integer(String),
    Float(String),
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FSROrinStr {
    Integer(String, Option<SingleOp>),
    Float(String, Option<SingleOp>),
    String(String),
}

impl FSROrinStr {
    pub fn to_2(&self) -> FSROrinStr2 {
        match self {
            FSROrinStr::Integer(i, op) => FSROrinStr2::Integer(i.to_string(), *op),
            FSROrinStr::Float(f, op) => FSROrinStr2::Float(f.to_string(), *op),
            FSROrinStr::String(s) => FSROrinStr2::String(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FSROrinStr2 {
    Integer(String, Option<SingleOp>),
    Float(String, Option<SingleOp>),
    String(String),
}

pub struct FSRFormatStruct {
    pub format_str: String,
    pub arg_strings: Vec<FSRToken>,
}

#[derive(Debug, Clone)]
pub enum FSRConstType {
    Normal,
    FormatString(FSRFormatStringInst),
    RegexString,
}

#[derive(Debug, Clone)]
pub struct FSRConstant {
    const_str: FSROrinStr,
    constant: FSRConstantType,
    const_type: FSRConstType,
    pub(crate) len: usize,
    pub(crate) single_op: Option<SingleOp>,
    meta: FSRPosition,
}

impl FSRConstant {
    pub fn get_const_str(&self) -> &FSROrinStr {
        &self.const_str
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_constant(&self) -> &FSRConstantType {
        &self.constant
    }

    pub fn convert_str_type(
        type_str: &str,
        content: &str,
        meta: FSRPosition,
        context: &mut ASTContext,
    ) -> FSRConstType {
        match type_str {
            "f" => {
                let mut format_inst = FSRFormatStringInst::new(content.to_string());
                format_inst.parse(meta, context);
                FSRConstType::FormatString(format_inst)
            }
            "r" => FSRConstType::RegexString,
            _ => FSRConstType::Normal,
        }
    }

    pub fn from_str(s: &[u8], meta: FSRPosition, const_type: FSRConstType) -> Self {
        FSRConstant {
            constant: FSRConstantType::String(s.to_vec()),
            len: 0,
            const_type,
            single_op: None,
            meta,
            const_str: FSROrinStr::String(unsafe { std::str::from_utf8_unchecked(s) }.to_string()),
        }
    }

    pub fn from_float(meta: FSRPosition, s: &str, op: Option<SingleOp>) -> Self {
        FSRConstant {
            constant: FSRConstantType::Float(s.to_string()),
            len: 0,
            single_op: op,
            const_type: FSRConstType::Normal,
            meta,
            const_str: FSROrinStr::Float(s.to_string(), op),
        }
    }

    pub fn from_int(meta: FSRPosition, s: &str, op: Option<SingleOp>) -> Self {
        FSRConstant {
            constant: FSRConstantType::Integer(s.to_string()),
            len: 0,
            single_op: op,
            const_type: FSRConstType::Normal,
            meta,
            const_str: FSROrinStr::Integer(s.to_string(), op),
        }
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn deduction(&self) -> FSRType {
        match &self.constant {
            FSRConstantType::String(_) => FSRType::new("String"),
            FSRConstantType::Integer(_) => FSRType::new("Integer"),
            FSRConstantType::Float(_) => FSRType::new("Float"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormatPlaceholder {
    pub start: usize,
    pub end: usize,
    pub expr: FSRToken,
}

/// Parser for format strings
/// Example: "hello, {user}, you have {count()} new messages."
/// Example2: "hello, \{user\}, you have \{count()\} new messages.", escaped braces
#[derive(Debug, Clone)]
pub struct FSRFormatStringInst {
    pub format_str: String,
    pub arg_expr: Vec<FormatPlaceholder>,
}

#[derive(Debug, Clone)]
struct BracedExpr {
    start: usize,    // '{' 的索引
    end: usize,      // '}' 的索引（包含）
    content: String, // 不含最外层花括号的内容
}

impl FSRFormatStringInst {
    pub fn new(format_str: String) -> Self {
        FSRFormatStringInst {
            format_str,
            arg_expr: vec![],
        }
    }

    fn extract_braced_expressions(s: &str) -> Vec<BracedExpr> {
        let chars: Vec<char> = s.chars().collect();
        let mut res = Vec::new();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '{' {
                // 跳过转义 "{{"
                if i + 1 < chars.len() && chars[i + 1] == '{' {
                    i += 2;
                    continue;
                }

                let start = i;
                i += 1;
                let mut depth = 1usize;
                let mut buf = String::new();
                let mut in_sq = false;
                let mut in_dq = false;
                let mut escape = false;

                while i < chars.len() {
                    let c = chars[i];

                    if escape {
                        buf.push(c);
                        escape = false;
                        i += 1;
                        continue;
                    }

                    if (in_sq || in_dq) && c == '\\' {
                        escape = true;
                        buf.push(c);
                        i += 1;
                        continue;
                    }

                    match c {
                        '\'' if !in_dq => {
                            in_sq = !in_sq;
                            buf.push(c);
                        }
                        '"' if !in_sq => {
                            in_dq = !in_dq;
                            buf.push(c);
                        }
                        '{' if !in_sq && !in_dq => {
                            depth += 1;
                            buf.push(c);
                        }
                        '}' if !in_sq && !in_dq => {
                            depth -= 1;
                            if depth == 0 {
                                // 完整表达式闭合
                                res.push(BracedExpr {
                                    start,
                                    end: i,
                                    content: buf,
                                });
                                break;
                            } else {
                                buf.push(c);
                            }
                        }
                        _ => buf.push(c),
                    }

                    i += 1;
                }
            } else if chars[i] == '}' {
                // 跳过转义 "}}"
                if i + 1 < chars.len() && chars[i + 1] == '}' {
                    i += 2;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        res
    }

    pub fn parse(
        &mut self,
        meta: FSRPosition,
        context: &mut ASTContext,
    ) -> Result<(), SyntaxError> {
        // Process the format string
        // like "hello, {user}, you have {count()} new messages."
        // like "hello {process_inner("sdf")}", inner braces can be treated as expressions
        let extracted = FSRFormatStringInst::extract_braced_expressions(&self.format_str);
        for expr in extracted {
            // Here we would normally parse the expression into an FSRToken
            // For demonstration, we will just create a placeholder FSRToken
            let token = FSRExpr::parse(
                expr.content.as_bytes(),
                true,
                meta.new_offset(expr.start),
                context,
            )?;
            self.arg_expr.push(FormatPlaceholder {
                start: expr.start,
                end: expr.end,
                expr: token.0,
            });
        }

        Ok(())
    }
}

#[test]
fn test_format() {
    let s = r#""hello {process_inner("s\"df")}, escaped {{ not expr }}, nested {a + 1}" "#;
    let extracted = FSRFormatStringInst::extract_braced_expressions(s);
    for (i, e) in extracted.iter().enumerate() {
        println!("{}: {}", i, e.content);
    }
}

#[test]
fn test_format_parser() {
    let s = r#""hello {process_inner("s\"df")}, escaped {{ not expr }}, nested {a + 1}" "#;
    let mut parser = FSRFormatStringInst::new(s.to_string());
    let mut context = ASTContext::new_context();
    let meta = FSRPosition::new();
    parser.parse(meta, &mut context).unwrap();
    for arg in parser.arg_expr.iter() {
        println!("Expr from {} to {}: {:#?}", arg.start, arg.end, arg.expr);
    }
}
