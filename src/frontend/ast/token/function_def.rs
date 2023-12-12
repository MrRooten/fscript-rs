use std::collections::HashMap;

use super::{base::FSRToken, hashtable::FSRHashtable};

pub struct FSRFunctionDef<'a> {
    name        : &'a str,
    args        : Vec<&'a str>,
    body        : Vec<FSRToken<'a>>,
    defaults    : Vec<FSRToken<'a>>
}

