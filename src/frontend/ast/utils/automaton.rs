use crate::backend::base_type::utils::i_to_m;
use std::collections::HashMap;

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
    ImportState
}

struct Node {
    id: u32,
    value: char,
    end_type: NodeType,
    subs: HashMap<char, Box<Node>>,
}

impl Node {
    pub fn get_node(&mut self, c: &char) -> Option<&Box<Node>> {
        return self.subs.get(c);
    }
}

pub struct FSTrie {
    root: Box<Node>,
    self_inc: u32,
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
        return s;
    }

    pub fn match_token(&mut self, token: &[u8]) -> Option<&NodeType> {
        let mut cur = &self.root;
        for c in token {
            let c = c.clone();
            if (c as char).is_ascii_alphabetic() == false {
                break;
            }
            let node = i_to_m(cur).get_node(&(c as char));
            let s = match node {
                Some(s) => i_to_m(s),
                None => {
                    return None;
                }
            };

            cur = s;
        }

        if cur.end_type == NodeType::NotEnd {
            return None;
        }
        return Some(&cur.end_type);
    }

    pub fn insert(&mut self, value: &str, n_type: NodeType) {
        let mut cur = &self.root;
        for c in value.chars() {
            let node = i_to_m(cur).get_node(&c);
            let s = match node {
                Some(s) => i_to_m(s),
                None => {
                    self.self_inc += 1;
                    let new_node = Node {
                        id: self.self_inc,
                        value: c,
                        end_type: NodeType::NotEnd,
                        subs: Default::default(),
                    };
                    let node = Box::new(new_node);
                    i_to_m(cur).subs.insert(c, node);
                    i_to_m(cur).subs.get_mut(&c).unwrap()
                }
            };

            cur = s;

        }
        i_to_m(cur).end_type = n_type.clone();
    }
}
