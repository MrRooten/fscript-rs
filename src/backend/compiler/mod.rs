use std::{collections::{HashMap, LinkedList}, sync::atomic::{AtomicU64, Ordering}};

use crate::frontend::ast::token::{base::FSRToken, expr::FSRExpr, variable::FSRVariable};

pub mod bytecode;
