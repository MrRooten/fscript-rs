use std::collections::HashMap;

use crate::frontend::ast::parse::ASTParser;

#[derive(PartialEq, Debug, Clone)]
pub enum NodeType {
    Root,
    NotEnd,
    Assign,
    IfState,
    WhileState,
    ForState,
    ExprState,
    FnState,
    ClassState,
    ReturnState,
    ImportState,
    Else,
    Break,
    Continue,
    Import,
    Try,
    Telling,
    Struct
}

#[allow(unused)]
struct Node {
    id: u32,
    value: char,
    end_type: NodeType,
    subs: HashMap<char, Box<Node>>,
}

impl Node {
    pub fn get_node(&mut self, c: &char) -> Option<&mut Box<Node>> {
        self.subs.get_mut(c)
    }

    pub fn get_subs(&mut self) -> &mut HashMap<char, Box<Node>> {
        &mut self.subs
    }
}

pub struct FSTrie {
    root: Box<Node>,
    self_inc: u32,
}

impl Default for FSTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl FSTrie {
    fn init(&mut self) {
        self.insert("let", NodeType::Assign);
        self.insert("if", NodeType::IfState);
        self.insert("while", NodeType::WhileState);
        self.insert("for", NodeType::ForState);
        self.insert("fn", NodeType::FnState);
        self.insert("class", NodeType::ClassState);
        self.insert("return", NodeType::ReturnState);
        self.insert("import", NodeType::ImportState);
        self.insert("else", NodeType::Else);
        self.insert("break", NodeType::Break);
        self.insert("continue", NodeType::Continue);
        self.insert("import", NodeType::Import);
        self.insert("try", NodeType::Try);
        self.insert("@", NodeType::FnState);
        self.insert("struct", NodeType::Struct);
    }

    pub fn new() -> FSTrie {
        let root = Node {
            id: 0,
            value: '\0',
            end_type: NodeType::Root,
            subs: Default::default(),
        };
        let mut s = Self {
            root: Box::new(root),
            self_inc: 0,
        };
        s.init();
        s
    }

    pub fn match_token(&mut self, token: &[u8]) -> Option<&NodeType> {
        let mut cur = &mut self.root;

        if token[0] == b'@' {
            return Some(&NodeType::FnState)
        }

        for c in token {
            let c = *c;
            let t_c = c as char;

            
            if !(c as char).is_ascii_alphabetic() {
                if ASTParser::is_name_letter(c) {
                    return None;
                }
                break;
            }
            let node = cur.get_node(&(c as char));
            let s = match node {
                Some(s) => s,
                None => {
                    return None;
                }
            };

            cur = s;
        }

        if cur.end_type == NodeType::NotEnd {
            return None;
        }
        Some(&cur.end_type)
    }

    pub fn insert(&mut self, value: &str, n_type: NodeType) {
        let mut cur = Some(&mut self.root);
        for c in value.chars() {
            // let node = cur.get_node(&c);
            let subs = cur.unwrap().get_subs();
            let node = subs.get(&c);
            match node {
                Some(_) => {}
                None => {
                    self.self_inc += 1;
                    let new_node = Node {
                        id: self.self_inc,
                        value: c,
                        end_type: NodeType::NotEnd,
                        subs: Default::default(),
                    };
                    let node = Box::new(new_node);
                    subs.insert(c, node);
                    //subs.get_mut(&c).unwrap()
                }
            };
            let s = subs.get_mut(&c);
            cur = s;
        }

        if let Some(s) = cur { 
            s.end_type = n_type.clone() 
        }
    }
}


mod test {
    use super::FSTrie;

    #[test]
    fn test_mod() {
        let mut t = FSTrie::new();
        t.init();

        assert!(t.match_token("fn_abc".as_bytes()).eq(&None));
    }
}