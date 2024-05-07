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
    pub fn get_node(&mut self, c: &char) -> Option<&mut Box<Node>> {
        return self.subs.get_mut(c);
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
        for c in token {
            let c = *c;
            if !(c as char).is_ascii_alphabetic() {
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
                Some(s) => {},
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

        match cur {
            Some(s) => s.end_type = n_type.clone(),
            None => {

            }
        }
    }
}
