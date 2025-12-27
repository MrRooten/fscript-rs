use std::{
    cell::Cell,
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    backend::types::base::ObjId,
    frontend::ast::token::{
        assign::FSRAssign,
        base::{FSRPosition, FSRToken, FSRType},
        block::FSRBlock,
        call::FSRCall,
        class::FSRClassFrontEnd,
        constant::{FSRConstType, FSRConstant, FSROrinStr2},
        expr::{FSRExpr, SingleOp},
        for_statement::FSRFor,
        function_def::FSRFnDef,
        if_statement::FSRIf,
        import::FSRImport,
        list::FSRListFrontEnd,
        module::FSRModuleFrontEnd,
        return_def::FSRReturn,
        slice::FSRGetter,
        try_expr::FSRTryBlock,
        variable::FSRVariable,
        while_statement::FSRWhile,
    },
};

macro_rules! ensure_attr_id {
    ($var_map:expr, $name:expr) => {{
        if !$var_map.last_mut().unwrap().has_attr($name) {
            $var_map.last_mut().unwrap().insert_attr($name);
        }
        *$var_map.last_mut().unwrap().get_attr($name).unwrap()
    }};
}

macro_rules! ensure_var_id {
    ($var_map:expr, $name:expr) => {{
        if !$var_map.last_mut().unwrap().has_var($name) {
            $var_map.last_mut().unwrap().insert_var($name);
        }
        *$var_map.last_mut().unwrap().get_var($name).unwrap()
    }};
}

macro_rules! ensure_const_id {
    ($var_map:expr, $c:expr) => {{
        let key = $c.to_2();
        if !$var_map.last().unwrap().const_map.contains_key(&key) {
            let r = if $var_map.last().unwrap().const_map.is_empty() {
                1
            } else {
                *$var_map.last().unwrap().const_map.values().max().unwrap() + 1
            };
            $var_map
                .last_mut()
                .unwrap()
                .const_map
                .insert(key.clone(), r);
        }
        *$var_map.last().unwrap().const_map.get(&key).unwrap()
    }};
}

pub struct StackVecMap {
    map: Vec<VarMap>,
}

impl StackVecMap {
    pub fn new_vec_map() -> Self {
        Self { map: vec![] }
    }

    pub fn push(&mut self, map: VarMap) {
        self.map.push(map);
    }

    pub fn pop(&mut self) -> Option<VarMap> {
        self.map.pop()
    }

    pub fn last_mut(&mut self) -> Option<&mut VarMap> {
        self.map.last_mut()
    }

    pub fn last(&self) -> Option<&VarMap> {
        self.map.last()
    }
}

#[derive(Debug, Clone)]
pub struct FSRPos {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct FSRPosHuman {
    pub line: usize,
    pub column: usize,
}

impl FSRPos {
    pub fn as_human(&self) -> FSRPosHuman {
        FSRPosHuman {
            line: self.line + 1,
            column: self.column + 1,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum BinaryOffset {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Greater = 3,
    GreatEqual = 4,
    Less = 5,
    LessEqual = 6,
    Equal = 7,
    NotEqual = 8,
    NextObject = 9,
    GetItem = 10,
    SetItem = 11,
    Div = 12,
    Index = 13,
    Order = 14,
    Hash = 15,
    Reminder = 16,
}

impl BinaryOffset {
    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn alias_name(&self) -> &'static str {
        match self {
            BinaryOffset::Add => "__add__",
            BinaryOffset::Sub => "__sub__",
            BinaryOffset::Mul => "__mul__",
            BinaryOffset::Greater => "__gt__",
            BinaryOffset::GreatEqual => "__gte__",
            BinaryOffset::Less => "__lt__",
            BinaryOffset::LessEqual => "__lte__",
            BinaryOffset::Equal => "__eq__",
            BinaryOffset::NotEqual => "__neq__",
            BinaryOffset::NextObject => "__next__",
            BinaryOffset::GetItem => "__get__",
            BinaryOffset::SetItem => "__set__",
            BinaryOffset::Div => "__div__",
            BinaryOffset::Index => "__index__",
            BinaryOffset::Order => "__ord__",
            BinaryOffset::Hash => "__hash__",
            BinaryOffset::Reminder => "__reminder__",
        }
    }

    pub fn from_alias_name(name: &str) -> Option<Self> {
        match name {
            "__add__" => Some(BinaryOffset::Add),
            "__sub__" => Some(BinaryOffset::Sub),
            "__mul__" => Some(BinaryOffset::Mul),
            "__gt__" => Some(BinaryOffset::Greater),
            "__gte__" => Some(BinaryOffset::GreatEqual),
            "__lt__" => Some(BinaryOffset::Less),
            "__lte__" => Some(BinaryOffset::LessEqual),
            "__eq__" => Some(BinaryOffset::Equal),
            "__neq__" => Some(BinaryOffset::NotEqual),
            "__next__" => Some(BinaryOffset::NextObject),
            "__get__" => Some(BinaryOffset::GetItem),
            "__set__" => Some(BinaryOffset::SetItem),
            "__div__" => Some(BinaryOffset::Div),
            "__index__" => Some(BinaryOffset::Index),
            "__ord__" => Some(BinaryOffset::Order),
            "__hash__" => Some(BinaryOffset::Hash),
            "__reminder__" => Some(BinaryOffset::Reminder),
            _ => None,
        }
    }
}

impl From<BinaryOffset> for usize {
    fn from(val: BinaryOffset) -> Self {
        val as usize
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum BytecodeOperator {
    Assign = 0,
    BinaryAdd = 1,
    BinaryDot = 2,
    BinaryMul = 3,
    Call = 4,
    IfTest = 5,
    WhileTest = 6,
    DefineFn = 7,
    EndFn = 8,
    CompareTest = 9,
    ReturnValue = 10,
    WhileBlockEnd = 11,
    AssignArgs = 12,
    ClassDef = 13,
    EndDefineClass = 14,
    LoadList = 15,
    Else = 16,
    ElseIf = 17,
    ElseIfTest = 18,
    IfBlockEnd = 19,
    Break = 20,
    Continue = 21,
    LoadForIter = 22,
    PushForNext = 24, // call iter_obj.__next__()
    ForBlockEnd = 23,
    SpecialLoadFor = 25,
    AndJump = 26,
    OrJump = 27,
    Empty = 28,
    // BinarySub,
    BinaryRShift = 29,
    BinaryLShift = 30,
    StoreFast = 31,
    BinarySub = 32,
    Import = 33,
    NotOperator = 34,
    BinaryDiv = 35,
    BinaryClassGetter = 36,
    Getter = 37,
    Try = 38,
    EndTry = 39,
    EndCatch = 40,
    BinaryRange = 41, // For -> operator
    // Add ref for loop like
    // for i in [1, 2, 3] {
    //
    //}
    // the [1, 2, 3] need to be ref
    ForBlockRefAdd = 42,
    /// Load current function
    /// use in nested function
    LoadSelfFn = 43,
    LoadConst = 44,
    BinaryReminder = 45,
    Create = 46,
    AssignContainer = 47,
    AssignAttr = 48,
    CallMethod = 49,
    CompareEqual = 50,
    TryException = 51,
    Await = 52, // await expression
    Yield = 53, // yield expression
    FormatString = 54,
    Delegate = 55,
    LoadYield = 56,
    OpAssign = 57,
    CLoadIntU8 = 58, // Only use for jit
    SLoadStr = 59,   // Only use for jit
    SLoadIntU16 = 60,
    SLoadIntU32 = 61,
    SLoadIntU64 = 62,
    SLoadArray = 63,
    Load = 254,
}

impl BytecodeOperator {
    pub fn from_u8(val: u8) -> Option<Self> {
        use BytecodeOperator::*;
        match val {
            0 => Some(Assign),
            1 => Some(BinaryAdd),
            2 => Some(BinaryDot),
            3 => Some(BinaryMul),
            4 => Some(Call),
            5 => Some(IfTest),
            6 => Some(WhileTest),
            7 => Some(DefineFn),
            8 => Some(EndFn),
            9 => Some(CompareTest),
            10 => Some(ReturnValue),
            11 => Some(WhileBlockEnd),
            12 => Some(AssignArgs),
            13 => Some(ClassDef),
            14 => Some(EndDefineClass),
            15 => Some(LoadList),
            16 => Some(Else),
            17 => Some(ElseIf),
            18 => Some(ElseIfTest),
            19 => Some(IfBlockEnd),
            20 => Some(Break),
            21 => Some(Continue),
            22 => Some(LoadForIter),
            23 => Some(ForBlockEnd),
            24 => Some(PushForNext),
            25 => Some(SpecialLoadFor),
            26 => Some(AndJump),
            27 => Some(OrJump),
            28 => Some(Empty),
            29 => Some(BinaryRShift),
            30 => Some(BinaryLShift),
            31 => Some(StoreFast),
            32 => Some(BinarySub),
            33 => Some(Import),
            34 => Some(NotOperator),
            35 => Some(BinaryDiv),
            36 => Some(BinaryClassGetter),
            37 => Some(Getter),
            38 => Some(Try),
            39 => Some(EndTry),
            40 => Some(EndCatch),
            41 => Some(BinaryRange),
            42 => Some(ForBlockRefAdd),
            43 => Some(LoadSelfFn),
            44 => Some(LoadConst),
            45 => Some(BinaryReminder),
            46 => Some(Create),
            47 => Some(AssignContainer),
            48 => Some(AssignAttr),
            49 => Some(CallMethod),
            254 => Some(Load),
            _ => None,
        }
    }
}

pub struct SArgWrapper {
    fallback: Box<ArgType>, // Used for alter for not jit
}

#[repr(C)]
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum CompareOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

impl CompareOperator {
    pub fn new_from_str(op: &str) -> Option<Self> {
        match op {
            "==" => Some(CompareOperator::Equal),
            "!=" => Some(CompareOperator::NotEqual),
            ">" => Some(CompareOperator::Greater),
            ">=" => Some(CompareOperator::GreaterEqual),
            "<" => Some(CompareOperator::Less),
            "<=" => Some(CompareOperator::LessEqual),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OpAssign {
    Add,
    Sub,
    Mul,
    Div,
    Reminder,
}

impl OpAssign {
    pub fn from_str(op: &str) -> Option<Self> {
        match op {
            "+=" => Some(OpAssign::Add),
            "-=" => Some(OpAssign::Sub),
            "*=" => Some(OpAssign::Mul),
            "/=" => Some(OpAssign::Div),
            "%=" => Some(OpAssign::Reminder),
            _ => None,
        }
    }

    pub fn get_offset(&self) -> BinaryOffset {
        match self {
            OpAssign::Add => BinaryOffset::Add,
            OpAssign::Sub => BinaryOffset::Sub,
            OpAssign::Mul => BinaryOffset::Mul,
            OpAssign::Div => BinaryOffset::Div,
            OpAssign::Reminder => BinaryOffset::Reminder,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalVar {
    // (u64, String, bool, Option<OpAssign>)
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) store_to_cell: bool,
    pub(crate) op_assign: Option<OpAssign>,
    pub(crate) var_type: Option<FSRSType>,
}

impl LocalVar {
    pub fn new(id: u64, name: String, store_to_cell: bool, op_assign: Option<OpAssign>) -> Self {
        Self {
            id,
            name: name,
            store_to_cell,
            op_assign,
            var_type: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClosureVar {
    // (u64, String, Option<OpAssign>)
    id: u64,
    name: String,
    op_assign: Option<OpAssign>,
}

#[derive(Debug, Clone)]
pub enum ArgType {
    Local(LocalVar),
    Global(String),
    ClosureVar((u64, String, Option<OpAssign>)),
    CurrentFn,
    Lambda((u64, String)),
    ImportModule(u64, Vec<String>),
    VariableList(Vec<(u64, String)>),
    ConstInteger(u64, i64, Option<SingleOp>),
    ConstFloat(u64, String, Option<SingleOp>),
    ConstString(u64, String),
    Const(u64),
    Attr(u64, String, Option<OpAssign>),
    BinaryOperator(BinaryOffset),
    IfTestNext((u64, u64)), // first u64 for if line, second for count else if /else
    WhileTest(u64),         //i64 is return to test, u64 is skip the block,
    WhileEnd(i64),
    Compare(CompareOperator),
    OpAssign((OpAssign, Box<ArgType>)),
    FnLines(usize),
    CallArgsNumber(usize),
    CallArgsNumberWithVar((usize, u64, String, bool)), // number size, Variable
    CallArgsNumberWithAttr((usize, u64, String)),
    DefineFnArgs(u64, String, String, Vec<String>, bool), // function len, args len, identify function name
    DefineClassLine(u64),
    LoadListNumber(usize),
    ForEnd(i64),
    AddOffset(usize),
    ForLine(u64),
    StoreFastVar(u64, String),
    Import(Vec<String>),
    TryCatch(u64, u64), // first u64 for catch start, second for catch end + 1
    GlobalId(ObjId),    // only for global id, like key object
    FnName(u64, String),
    ClassName(u64, String),
    LoadTrue,
    LoadFalse,
    LoadNone,
    FormatStringLen(u64, String), // length, format string
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum FSRDbgFlag {
    None,
    Keep,
    Once,
}

#[derive(Debug, Clone)]
pub struct FSRByteInfo {
    pos: FSRPos,
    dbg_flag: Cell<FSRDbgFlag>,
}

impl FSRByteInfo {
    pub fn get_pos(&self) -> &FSRPos {
        &self.pos
    }

    /// Create a new FSRByteInfo from lines and meta position.
    /// # Arguments
    /// * `lines` - A vector of usize representing the lines in the source code.
    /// * `meta` - A FSRPosition representing the position in the source code.
    /// # Returns
    /// * `FSRByteInfo` - A new instance of FSRByteInfo with the position set.
    pub fn new(lines: &[usize], meta: FSRPosition) -> Self {
        if lines.is_empty() {
            return Self {
                pos: FSRPos {
                    line: 0,
                    column: meta.get_offset(),
                },
                dbg_flag: Cell::new(FSRDbgFlag::None),
            };
        }

        let offset = meta.get_offset();
        let mut left = 0;
        let mut right = 1;

        if offset <= lines[0] {
            return Self {
                pos: FSRPos {
                    line: 0,
                    column: offset,
                },
                dbg_flag: Cell::new(FSRDbgFlag::None),
            };
        }

        while right < lines.len() && lines[right] <= offset {
            left += 1;
            right += 1;
        }

        let pos = FSRPos {
            line: left + 1,
            column: offset - lines[left],
        };

        Self {
            pos,
            dbg_flag: Cell::new(FSRDbgFlag::None),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BytecodeArg {
    operator: BytecodeOperator,
    /// Change 'arg: ArgType' to 'arg: Box<ArgType>'
    arg: Box<ArgType>,
    info: Box<FSRByteInfo>,
}

impl BytecodeArg {
    pub fn get_pos(&self) -> FSRPos {
        self.info.pos.clone()
    }

    #[inline]
    pub fn get_operator(&self) -> &BytecodeOperator {
        &self.operator
    }

    #[inline]
    pub fn get_arg(&self) -> &ArgType {
        &self.arg
    }

    pub fn is_dbg(&self) -> bool {
        match self.info.dbg_flag.get() {
            FSRDbgFlag::None => false,
            FSRDbgFlag::Keep => true,
            FSRDbgFlag::Once => true,
        }
    }

    pub fn set_dbg(&self, dbg_flag: FSRDbgFlag) {
        self.info.dbg_flag.set(dbg_flag);
    }

    pub fn is_dbg_once(&self) -> bool {
        match self.info.dbg_flag.get() {
            FSRDbgFlag::None => false,
            FSRDbgFlag::Keep => true,
            FSRDbgFlag::Once => false,
        }
    }
}

impl BytecodeOperator {
    pub fn get_static_op(op: &str) -> &'static str {
        // op reference my not life longer enough, so return static str
        if op.eq(">") {
            return ">";
        } else if op.eq("<") {
            return "<";
        } else if op.eq(">=") {
            return ">=";
        } else if op.eq("<=") {
            return "<=";
        } else if op.eq("==") {
            return "==";
        } else if op.eq(".") {
            return ".";
        } else if op.eq("!=") {
            return "!=";
        } else if op.eq("::") {
            return "::";
        } else if op.eq(":") {
            return ":";
        } else if op.eq("+=") {
            return "+=";
        } else if op.eq("-=") {
            return "-=";
        } else if op.eq("*=") {
            return "*=";
        } else if op.eq("/=") {
            return "/=";
        } else if op.eq("%=") {
            return "%=";
        }

        unimplemented!()
    }

    pub fn get_op(
        op: &str,
        info: Box<FSRByteInfo>,
        attr_id: Option<ArgType>,
    ) -> Option<BytecodeArg> {
        if op.eq("+") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryAdd,
                arg: Box::new(ArgType::None),
                info,
            });
        } else if op.eq("*") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryMul,
                arg: Box::new(ArgType::None),
                info,
            });
        } else if op.eq(".") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryDot,
                arg: Box::new(attr_id.unwrap()),
                info,
            });
        } else if op.eq("::") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryClassGetter,
                arg: Box::new(attr_id.unwrap()),
                info,
            });
        } else if op.eq("=") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::Assign,
                arg: Box::new(ArgType::None),
                info,
            });
        } else if op.eq(">")
            || op.eq("<")
            || op.eq(">=")
            || op.eq("<=")
            || op.eq("==")
            || op.eq("!=")
        {
            return Some(BytecodeArg {
                operator: BytecodeOperator::CompareTest,
                arg: Box::new(ArgType::Compare(
                    CompareOperator::new_from_str(op)
                        .expect(&format!("Invalid compare operator: {}", op)),
                )),
                info,
            });
        } else if op.eq("<<") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryLShift,
                arg: Box::new(ArgType::None),
                info,
            });
        } else if op.eq(">>") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryRShift,
                arg: Box::new(ArgType::None),
                info,
            });
        } else if op.eq("-") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinarySub,
                arg: Box::new(ArgType::None),
                info,
            });
        } else if op.eq("/") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryDiv,
                arg: Box::new(ArgType::None),
                info,
            });
        } else if op.eq("..") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryRange,
                arg: Box::new(ArgType::None),
                info,
            });
        } else if op.eq("%") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryReminder,
                arg: Box::new(ArgType::None),
                info,
            });
        }
        // } else if op.eq("&&") || op.eq("and") {
        //     return Some(BytecodeArg {
        //         operator: BytecodeOperator::LoadAnd,
        //         arg: Box::new(ArgType::None)
        //     })
        // } else if op.eq("||") || op.eq("or") {
        //     return Some(BytecodeArg {
        //         operator: BytecodeOperator::LoadOr,
        //         arg: Box::new(ArgType::None)
        //     })
        // }
        None
    }
}

#[derive(Debug)]
pub struct FnDef {
    code: Vec<Vec<BytecodeArg>>,
    var_map: VarMap,
    is_jit: bool,
    is_async: bool,
    is_static: bool,
}

#[derive(Debug, Clone)]
pub enum FSRSType {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    IInt8,
    IInt16,
    IInt32,
    IInt64,
    Float32,
    Float64,
    String,
    Array,
    Other,
}

#[derive(Debug)]
pub struct FSRSTypeInfo {}

impl FSRSTypeInfo {
    pub fn get_type(&self, name: &str) -> Option<FSRSType> {
        match name {
            "u8" => Some(FSRSType::UInt8),
            "u16" => Some(FSRSType::UInt16),
            "u32" => Some(FSRSType::UInt32),
            "u64" => Some(FSRSType::UInt64),
            "i8" => Some(FSRSType::IInt8),
            "i16" => Some(FSRSType::IInt16),
            "i32" => Some(FSRSType::IInt32),
            "i64" => Some(FSRSType::IInt64),
            "f32" => Some(FSRSType::Float32),
            "f64" => Some(FSRSType::Float64),
            "string" => Some(FSRSType::String),
            "array" => Some(FSRSType::Array),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct BytecodeContext {
    //pub(crate) const_map: HashMap<FSROrinStr2, u64>,
    pub(crate) table: Vec<ObjId>,
    pub(crate) fn_def_map: HashMap<String, FnDef>,
    pub(crate) ref_map_stack: Vec<HashMap<String, bool>>,
    pub(crate) cur_fn_name: Vec<String>,
    pub(crate) key_map: HashMap<&'static str, ArgType>,
    pub(crate) lines: Vec<usize>,
    pub(crate) is_static: bool,
    pub(crate) type_info: FSRSTypeInfo,
}

#[allow(clippy::new_without_default)]
impl BytecodeContext {
    pub fn new(lines: Vec<usize>) -> Self {
        let mut v = HashMap::new();
        v.insert("true", ArgType::LoadTrue);
        v.insert("false", ArgType::LoadFalse);
        v.insert("none", ArgType::LoadNone);
        Self {
            //const_map: HashMap::new(),
            table: vec![0],
            fn_def_map: HashMap::new(),
            ref_map_stack: vec![],
            cur_fn_name: vec![],
            key_map: v,
            lines,
            is_static: false,
            type_info: FSRSTypeInfo {},
        }
    }

    pub fn insert_table(&mut self, c_id: usize, obj_id: ObjId) {
        if c_id + 1 > self.table.len() {
            self.table.resize(c_id + 1, 0);
        }

        self.table[c_id] = obj_id;
    }

    pub fn get_from_table(&self, c_id: usize) -> Option<ObjId> {
        let v = self.table.get(c_id).cloned();
        match v {
            Some(0) => None,
            Some(v) => Some(v),
            None => None,
        }
    }

    pub fn contains_variable_in_ref_stack(&self, name: &str) -> bool {
        for i in &self.ref_map_stack {
            if let Some(v) = i.get(name) {
                return *v;
            }
        }

        false
    }

    /// Check if the variable is in the reference map stack, but not in the last one.
    /// This is useful for checking if a variable is defined in the current scope but not in the last scope.
    /// If the stack has less than 2 elements, it returns false.
    ///
    /// # Arguments
    /// * `name` - The name of the variable to check.
    ///
    /// # Returns
    /// * `bool` - Returns true if the variable is found in any of the reference maps except the last one, otherwise false.
    ///
    pub fn contains_variable_in_ref_stack_not_last(&self, name: &str) -> bool {
        if self.ref_map_stack.len() < 2 {
            return false;
        }
        for i in &self.ref_map_stack[..self.ref_map_stack.len() - 1] {
            if let Some(v) = i.get(name) {
                return *v;
            }
        }

        false
    }

    pub fn contains_in_cur_ref(&self, name: &str) -> bool {
        if let Some(ref_map) = self.ref_map_stack.last() {
            if let Some(v) = ref_map.get(name) {
                return *v;
            }
        }
        false
    }

    pub fn variable_is_defined(&self, name: &str) -> bool {
        if let Some(ref_map) = self.ref_map_stack.last() {
            if let Some(_) = ref_map.get(name) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug)]
pub struct VarMap {
    pub(crate) var_map: HashMap<String, u64>,
    pub(crate) var_id: AtomicU64,
    pub(crate) attr_map: HashMap<String, u64>,
    pub(crate) attr_id: AtomicU64,
    pub(crate) const_map: HashMap<FSROrinStr2, u64>,
    pub(crate) const_id: AtomicU64,
    pub(crate) name: String,
    pub(crate) sub_fn_def: Vec<Bytecode>,
}

impl VarMap {
    pub fn has_var(&self, var: &str) -> bool {
        self.var_map.contains_key(var)
    }

    pub fn get_var(&self, var: &str) -> Option<&u64> {
        self.var_map.get(var)
    }

    pub fn insert_var(&mut self, var: &str) {
        if self.var_map.contains_key(var) {
            return;
        }
        let v = self.var_id.fetch_add(1, Ordering::Acquire);
        self.var_map.insert(var.to_owned(), v);
    }

    // pub fn has_const(&self, c: &FSROrinStr2) -> bool {
    //     self.const_map.contains_key(c)
    // }

    // pub fn get_const(&self, c: &FSROrinStr2) -> Option<u64> {
    //     self.const_map.get(c).copied()
    // }

    // pub fn insert_const(&mut self, c: &FSROrinStr2) {
    //     if self.has_const(c) {
    //         return;
    //     }
    //     let v = self.const_id.fetch_add(1, Ordering::Acquire);
    //     self.const_map.insert(*c, v);
    // }

    pub fn insert_attr(&mut self, attr: &str) {
        let v = self.attr_id.fetch_add(1, Ordering::Acquire);
        self.attr_map.insert(attr.to_owned(), v);
    }

    pub fn has_attr(&self, attr: &str) -> bool {
        self.attr_map.contains_key(attr)
    }

    pub fn get_attr(&self, attr: &str) -> Option<&u64> {
        self.attr_map.get(attr)
    }

    pub fn new(name: &str) -> Self {
        Self {
            var_map: HashMap::new(),
            var_id: AtomicU64::new(1),
            attr_map: HashMap::new(),
            attr_id: AtomicU64::new(1),
            const_map: HashMap::new(),
            const_id: AtomicU64::new(1),
            name: name.to_string(),
            sub_fn_def: vec![],
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Bytecode {
    pub(crate) name: String,
    pub(crate) context: BytecodeContext,
    pub(crate) bytecode: Vec<Vec<BytecodeArg>>,
    pub(crate) var_map: VarMap,
    pub(crate) is_jit: bool,
    pub(crate) is_async: bool,
}

enum AttrIdOrCode {
    Bytecode(Vec<BytecodeArg>),
    AttrId(ArgType),
}

impl<'a> Bytecode {
    pub fn get(&self, index: usize) -> Option<&Vec<BytecodeArg>> {
        if let Some(s) = self.bytecode.get(index) {
            return Some(s);
        }

        None
    }

    fn load_list_getter(
        getter: &FSRGetter,
        var_map: &mut Vec<VarMap>,
        is_attr: bool,
        is_method_call: bool,
        is_assign: bool,
        const_map: &mut BytecodeContext,
    ) -> (Vec<BytecodeArg>) {
        let mut result = Vec::new();
        let name = getter.get_name();
        if !name.is_empty() {
            if is_attr {
                if !var_map.last_mut().unwrap().has_attr(name) {
                    let v = name;
                    var_map.last_mut().unwrap().insert_attr(v);
                }
                let id = var_map.last_mut().unwrap().get_attr(name).unwrap();

                if is_method_call {
                    result.push(BytecodeArg {
                        operator: BytecodeOperator::BinaryDot,
                        arg: Box::new(ArgType::Attr(*id, name.to_string(), None)),
                        info: Box::new(FSRByteInfo::new(
                            &const_map.lines,
                            getter.get_meta().clone(),
                        )),
                    });
                } else {
                    result.push(BytecodeArg {
                        operator: BytecodeOperator::BinaryClassGetter,
                        arg: Box::new(ArgType::Attr(*id, name.to_string(), None)),
                        info: Box::new(FSRByteInfo::new(
                            &const_map.lines,
                            getter.get_meta().clone(),
                        )),
                    });
                }
            } else {
                let id = ensure_var_id!(var_map, name);
                if !getter.is_defined && const_map.contains_variable_in_ref_stack(getter.get_name())
                {
                    result.push(BytecodeArg {
                        operator: BytecodeOperator::Load,
                        arg: Box::new(ArgType::ClosureVar((id, name.to_string(), None))),
                        info: Box::new(FSRByteInfo::new(
                            &const_map.lines,
                            getter.get_meta().clone(),
                        )),
                    });
                } else {
                    result.push(BytecodeArg {
                        operator: BytecodeOperator::Load,
                        arg: Box::new(ArgType::Local(LocalVar::new(
                            id,
                            name.to_string(),
                            false,
                            None,
                        ))),
                        info: Box::new(FSRByteInfo::new(
                            &const_map.lines,
                            getter.get_meta().clone(),
                        )),
                    });
                }
            }
        }

        let mut v =
            Self::load_token_with_map(getter.get_getter(), var_map, const_map, false, false);
        result.append(&mut v[0]);

        if !is_assign {
            result.push(BytecodeArg {
                operator: BytecodeOperator::Getter,
                arg: Box::new(ArgType::None),
                info: Box::new(FSRByteInfo::new(
                    &const_map.lines,
                    getter.get_meta().clone(),
                )),
            });
        }

        (result)
    }

    fn load_call(
        call: &FSRCall,
        var_map: &mut Vec<VarMap>,
        is_attr: bool,
        is_method_call: bool,
        context: &mut BytecodeContext,
    ) -> (Vec<BytecodeArg>) {
        let mut result = Vec::new();

        let name = call.get_name();
        let mut is_var = false;
        let mut var_id = 0;
        let mut attr_id_arg = None;
        if !name.is_empty() {
            if is_attr {
                let id = ensure_attr_id!(var_map, name);
                attr_id_arg = Some((id, name.to_string()));
                if is_method_call {
                } else {
                    result.push(BytecodeArg {
                        operator: BytecodeOperator::BinaryClassGetter,
                        arg: Box::new(ArgType::Attr(id, name.to_string(), None)),
                        info: Box::new(FSRByteInfo::new(&context.lines, call.get_meta().clone())),
                    });
                }
            } else {
                let id = ensure_var_id!(var_map, name);
                // if !call.is_defined && const_map.contains_variable_in_ref_stack(call.get_name()) {
                if !context.cur_fn_name.is_empty() && name.eq(context.cur_fn_name.last().unwrap()) {
                    result.push(BytecodeArg {
                        operator: BytecodeOperator::Load,
                        arg: Box::new(ArgType::CurrentFn),
                        info: Box::new(FSRByteInfo::new(&context.lines, call.get_meta().clone())),
                    });
                } else if context.contains_variable_in_ref_stack_not_last(call.get_name()) {
                    result.push(BytecodeArg {
                        operator: BytecodeOperator::Load,
                        arg: Box::new(ArgType::ClosureVar((id, name.to_string(), None))),
                        info: Box::new(FSRByteInfo::new(&context.lines, call.get_meta().clone())),
                    });
                } else {
                    let arg =
                        if context.variable_is_defined(name) || context.ref_map_stack.is_empty() {
                            ArgType::Local(LocalVar::new(id, name.to_string(), false, None))
                        } else {
                            ArgType::Global(name.to_string())
                        };

                    result.push(BytecodeArg {
                        operator: BytecodeOperator::Load,
                        arg: Box::new(arg),
                        info: Box::new(FSRByteInfo::new(&context.lines, call.get_meta().clone())),
                    });
                }
            }
        }

        for arg in call.get_args() {
            let mut v = Self::load_token_with_map(arg, var_map, context, false, false);
            result.append(&mut v[0]);
        }

        let call_or_callmethod = if is_method_call {
            BytecodeOperator::CallMethod
        } else {
            BytecodeOperator::Call
        };

        let arg = if is_method_call {
            ArgType::CallArgsNumberWithAttr((
                call.get_args().len(),
                attr_id_arg.as_ref().unwrap().0,
                attr_id_arg.unwrap().1,
            ))
        } else {
            ArgType::CallArgsNumber(call.get_args().len())
        };

        // if is_var {
        result.push(BytecodeArg {
            operator: call_or_callmethod,
            arg: Box::new(arg),
            info: Box::new(FSRByteInfo::new(&context.lines, call.get_meta().clone())),
        });
        // } else {
        //     result.push(BytecodeArg {
        //         operator: call_or_callmethod,
        //         arg: ArgType::CallArgsNumber(call.get_args().len()),
        //         info: FSRByteInfo::new(&context.lines,call.get_meta().clone()),
        //     });
        // }

        (result)
    }

    fn load_variable(
        var: &FSRVariable,
        var_map: &mut Vec<VarMap>,
        is_attr: bool,
        context: &mut BytecodeContext,
    ) -> (AttrIdOrCode) {
        if !is_attr {
            if !var_map.last_mut().unwrap().has_var(var.get_name()) {
                let v = var.get_name();
                var_map.last_mut().unwrap().insert_var(v);
            }
        } else if !var_map.last_mut().unwrap().has_attr(var.get_name()) {
            let v = var.get_name();
            var_map.last_mut().unwrap().insert_attr(v);
        }

        if context.key_map.contains_key(var.get_name()) {
            let obj = context.key_map.get(var.get_name()).unwrap().clone();
            let op_arg = BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: Box::new(obj),
                info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
            };

            let mut ans = vec![op_arg];
            if let Some(single_op) = var.single_op {
                match single_op {
                    SingleOp::Not => {
                        ans.push(BytecodeArg {
                            operator: BytecodeOperator::NotOperator,
                            arg: Box::new(ArgType::None),
                            info: Box::new(FSRByteInfo::new(
                                &context.lines,
                                var.get_meta().clone(),
                            )),
                        });
                    }
                    _ => {
                        panic!("not support single op {:?}", single_op);
                    }
                }
            }

            return (AttrIdOrCode::Bytecode(ans));
        }

        if context.contains_variable_in_ref_stack(var.get_name()) && !var.is_defined {
            // if context.contains_variable_in_ref_stack(var.get_name()) && !var.is_defined{
            let op_arg = match is_attr {
                true => {
                    let arg_id = var_map
                        .last_mut()
                        .unwrap()
                        .get_attr(var.get_name())
                        .unwrap();
                    return AttrIdOrCode::AttrId(ArgType::Attr(
                        *arg_id,
                        var.get_name().to_string(),
                        None,
                    ));
                }
                false => BytecodeArg {
                    operator: BytecodeOperator::Load,
                    arg: {
                        let arg_id = var_map.last_mut().unwrap().get_var(var.get_name()).unwrap();
                        Box::new(ArgType::ClosureVar((
                            *arg_id,
                            var.get_name().to_string(),
                            None,
                        )))
                    },
                    info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
                },
            };
            let mut ans = vec![op_arg];
            if let Some(single_op) = var.single_op {
                match single_op {
                    SingleOp::Not => {
                        ans.push(BytecodeArg {
                            operator: BytecodeOperator::NotOperator,
                            arg: Box::new(ArgType::None),
                            info: Box::new(FSRByteInfo::new(
                                &context.lines,
                                var.get_meta().clone(),
                            )),
                        });
                    }
                    _ => {
                        panic!("not support single op {:?}", single_op);
                    }
                }
            }

            return (AttrIdOrCode::Bytecode(ans));
        }

        let op_arg = match is_attr {
            true => {
                let arg_id = var_map
                    .last_mut()
                    .unwrap()
                    .get_attr(var.get_name())
                    .unwrap();
                return AttrIdOrCode::AttrId(ArgType::Attr(
                    *arg_id,
                    var.get_name().to_string(),
                    None,
                ));
            }
            false => {
                let arg_id = var_map.last_mut().unwrap().get_var(var.get_name()).unwrap();
                let arg = if context.variable_is_defined(var.get_name()) {
                    ArgType::Local(LocalVar::new(
                        *arg_id,
                        var.get_name().to_string(),
                        false,
                        None,
                    ))
                } else {
                    ArgType::Global(var.get_name().to_string())
                };
                BytecodeArg {
                    operator: BytecodeOperator::Load,
                    arg: Box::new(arg),
                    info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
                }
            }
        };
        let mut ans = vec![op_arg];
        if let Some(single_op) = var.single_op {
            match single_op {
                SingleOp::Not => {
                    ans.push(BytecodeArg {
                        operator: BytecodeOperator::NotOperator,
                        arg: Box::new(ArgType::None),
                        info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
                    });
                }
                _ => {
                    panic!("not support single op {:?}", single_op);
                }
            }
        }

        (AttrIdOrCode::Bytecode(ans))
    }

    fn load_assign_arg(
        var: &'a FSRVariable,
        var_map: &mut Vec<VarMap>,
        context: &mut BytecodeContext,
    ) -> (Vec<BytecodeArg>) {
        if !var_map.last_mut().unwrap().has_var(var.get_name()) {
            let v = var.get_name();
            var_map.last_mut().unwrap().insert_var(v);
        }

        if let Some(ref_map) = context.ref_map_stack.last() {
            if ref_map.get(var.get_name()).cloned().unwrap_or(false) {
                let arg_id = var_map.last_mut().unwrap().get_var(var.get_name()).unwrap();
                let op_arg: BytecodeArg = BytecodeArg {
                    operator: BytecodeOperator::AssignArgs,
                    arg: Box::new(ArgType::ClosureVar((
                        *arg_id,
                        var.get_name().to_string(),
                        None,
                    ))),
                    info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
                };

                return vec![op_arg];
            }
        }

        let arg_id = var_map.last_mut().unwrap().get_var(var.get_name()).unwrap();

        let op_arg = BytecodeArg {
            operator: BytecodeOperator::AssignArgs,
            arg: Box::new(ArgType::Local(LocalVar::new(
                *arg_id,
                var.get_name().to_string(),
                false,
                None,
            ))),
            info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
        };

        let ans = vec![op_arg];

        ans
    }

    fn load_stack_expr(
        var: &(Option<SingleOp>, Vec<FSRToken>),
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
        mut is_attr: bool,
        mut is_method_call: bool,
    ) -> (Vec<BytecodeArg>) {
        let mut result = Vec::new();
        for token in var.1.iter() {
            let mut v =
                Self::load_token_with_map(token, var_map, const_map, is_attr, is_method_call);
            is_attr = false;
            is_method_call = false;
            if v.is_empty() {
                continue;
            }

            result.append(&mut v[0]);
        }

        (result)
    }

    /// Use to trigger special attribute
    /// like var.await, var.yield, var.return, var.throw
    fn special_variable_trigger(
        var: &FSRVariable,
        context: &mut BytecodeContext,
    ) -> Option<Vec<BytecodeArg>> {
        if var.get_name().eq("try") {
            return Some(vec![BytecodeArg {
                operator: BytecodeOperator::TryException,
                arg: Box::new(ArgType::None),
                info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
            }]);
        } else if var.get_name().eq("await") {
            return Some(vec![BytecodeArg {
                operator: BytecodeOperator::Await,
                arg: Box::new(ArgType::None),
                info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
            }]);
        } else if var.get_name().eq("yield") {
            return Some(vec![
                BytecodeArg {
                    operator: BytecodeOperator::Yield,
                    arg: Box::new(ArgType::None),
                    info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
                },
                BytecodeArg {
                    operator: BytecodeOperator::LoadYield,
                    arg: Box::new(ArgType::None),
                    info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
                },
            ]);
        } else if var.get_name().eq("delegate") {
            return Some(vec![BytecodeArg {
                operator: BytecodeOperator::Delegate,
                arg: Box::new(ArgType::None),
                info: Box::new(FSRByteInfo::new(&context.lines, var.get_meta().clone())),
            }]);
        }
        None
    }

    fn load_expr(
        expr: &FSRExpr,
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> (Vec<BytecodeArg>) {
        let mut op_code = Vec::new();
        if let FSRToken::Expr(sub_expr) = expr.get_left() {
            let mut v = Self::load_expr(sub_expr, var_map, const_map);
            op_code.append(&mut v);
        } else if let FSRToken::Variable(v) = expr.get_left() {
            let mut v = Self::load_variable(v, var_map, false, const_map);
            match v {
                AttrIdOrCode::Bytecode(mut bytecode_args) => {
                    op_code.append(&mut bytecode_args);
                }
                AttrIdOrCode::AttrId(arg_type) => todo!(),
            }
        } else if let FSRToken::Call(c) = expr.get_left() {
            let mut v = Self::load_call(c, var_map, false, false, const_map);
            op_code.append(&mut v);
        } else if let FSRToken::Getter(s) = expr.get_left() {
            let mut v = Self::load_list_getter(s, var_map, false, false, false, const_map);
            op_code.append(&mut v);
        } else if let FSRToken::Constant(c) = expr.get_left() {
            let mut v = Self::load_constant(c, var_map, const_map);
            op_code.append(&mut v);
        } else if let FSRToken::StackExpr(st) = expr.get_left() {
            let mut v = Self::load_stack_expr(st, var_map, const_map, false, false);
            op_code.append(&mut v);
        } else if let FSRToken::List(list) = expr.get_left() {
            let mut v = Self::load_list(list, var_map, const_map);
            op_code.append(&mut v);
        } else {
            println!("{:#?}", expr.get_left());
            unimplemented!()
        }

        let mut second = Vec::new();
        let mut attr_id = None;
        if let FSRToken::Expr(sub_expr) = expr.get_right() {
            let mut v = Self::load_expr(sub_expr, var_map, const_map);
            second.append(&mut v);
            //
        } else if let FSRToken::Variable(v) = expr.get_right() {
            let mut is_attr = false;

            if expr.get_op().eq(".") {
                let v = Self::special_variable_trigger(v, const_map);
                if let Some(op_arg) = v {
                    second.extend(op_arg);
                    op_code.append(&mut second);
                    return (op_code);
                }
            }

            if expr.get_op().eq(".") || expr.get_op().eq("::") {
                is_attr = true;
            }

            let v = Self::load_variable(v, var_map, is_attr, const_map);
            match v {
                AttrIdOrCode::Bytecode(mut bytecode_args) => {
                    second.append(&mut bytecode_args);
                }
                AttrIdOrCode::AttrId(arg_type) => attr_id = Some(arg_type),
            }

            //
        } else if let FSRToken::Call(c) = expr.get_right() {
            let mut is_attr = false;
            let mut is_method_call = false;
            if expr.get_op().eq(".") || expr.get_op().eq("::") {
                is_attr = true;
            }

            if expr.get_op().eq(".") {
                is_method_call = true;
            }

            //println!("call: {:#?}", expr);

            let mut v = Self::load_call(c, var_map, is_attr, is_method_call, const_map);
            second.append(&mut v);

            //call special process
            if expr.get_op().eq(".") || expr.get_op().eq("::") {
                op_code.append(&mut second);
                if let Some(single_op) = expr.get_single_op() {
                    if single_op.eq(&SingleOp::Not) {
                        op_code.push(BytecodeArg {
                            operator: BytecodeOperator::NotOperator,
                            arg: Box::new(ArgType::None),
                            info: Box::new(FSRByteInfo::new(
                                &const_map.lines,
                                expr.get_meta().clone(),
                            )),
                        });
                    } else {
                        panic!("not support this single op: {:?}", single_op);
                    }
                }
                return (op_code);
            }
        } else if let FSRToken::Getter(s) = expr.get_right() {
            let mut is_attr = false;
            let mut is_method_call = true;
            if expr.get_op().eq(".") || expr.get_op().eq("::") {
                is_attr = true;
            }

            if expr.get_op().eq("::") {
                is_method_call = false;
            }
            let mut v =
                Self::load_list_getter(s, var_map, is_attr, is_method_call, false, const_map);
            second.append(&mut v);

            //call special process
            if expr.get_op().eq(".") || expr.get_op().eq("::") {
                op_code.append(&mut second);
                if let Some(single_op) = expr.get_single_op() {
                    if single_op.eq(&SingleOp::Not) {
                        op_code.push(BytecodeArg {
                            operator: BytecodeOperator::NotOperator,
                            arg: Box::new(ArgType::None),
                            info: Box::new(FSRByteInfo::new(
                                &const_map.lines,
                                expr.get_meta().clone(),
                            )),
                        });
                    } else {
                        panic!("not support this single op: {:?}", single_op);
                    }
                }
                return (op_code);
            }
        } else if let FSRToken::Constant(c) = expr.get_right() {
            let mut v = Self::load_constant(c, var_map, const_map);
            second.append(&mut v);
            //
        } else if let FSRToken::StackExpr(st) = expr.get_right() {
            let mut is_attr = false;
            let mut is_method_call = true;
            if expr.get_op().eq(".") || expr.get_op().eq("::") {
                is_attr = true;
            }

            if expr.get_op().eq("::") {
                is_method_call = false;
            }
            let mut v = Self::load_stack_expr(st, var_map, const_map, is_attr, is_method_call);
            second.append(&mut v);
            if expr.get_op().eq(".") || expr.get_op().eq("::") {
                op_code.append(&mut second);
                if let Some(single_op) = expr.get_single_op() {
                    if single_op.eq(&SingleOp::Not) {
                        op_code.push(BytecodeArg {
                            operator: BytecodeOperator::NotOperator,
                            arg: Box::new(ArgType::None),
                            info: Box::new(FSRByteInfo::new(
                                &const_map.lines,
                                expr.get_meta().clone(),
                            )),
                        });
                    } else {
                        panic!("not support this single op: {:?}", single_op);
                    }
                }
                return (op_code);
            }

            second.append(&mut v);
        } else if let FSRToken::List(list) = expr.get_right() {
            let mut v = Self::load_list(list, var_map, const_map);
            second.append(&mut v);
            //op_code.append(&mut v);
            //
        } else {
            println!("{:#?}", expr.get_right());
            unimplemented!()
        }
        if expr.get_op().eq("&&") || expr.get_op().eq("and") {
            op_code.push(BytecodeArg {
                operator: BytecodeOperator::AndJump,
                arg: Box::new(ArgType::AddOffset(second.len())),
                info: Box::new(FSRByteInfo::new(&const_map.lines, expr.get_meta().clone())),
            });
            op_code.append(&mut second);
            if let Some(single_op) = expr.get_single_op() {
                if single_op.eq(&SingleOp::Not) {
                    op_code.push(BytecodeArg {
                        operator: BytecodeOperator::NotOperator,
                        arg: Box::new(ArgType::None),
                        info: Box::new(FSRByteInfo::new(&const_map.lines, expr.get_meta().clone())),
                    });
                } else {
                    panic!("not support this single op: {:?}", single_op);
                }
            }
            return (op_code);
        } else if expr.get_op().eq("||") || expr.get_op().eq("or") {
            op_code.push(BytecodeArg {
                operator: BytecodeOperator::OrJump,
                arg: Box::new(ArgType::AddOffset(second.len())),
                info: Box::new(FSRByteInfo::new(&const_map.lines, expr.get_meta().clone())),
            });
            op_code.append(&mut second);
            if let Some(single_op) = expr.get_single_op() {
                if single_op.eq(&SingleOp::Not) {
                    op_code.push(BytecodeArg {
                        operator: BytecodeOperator::NotOperator,
                        arg: Box::new(ArgType::None),
                        info: Box::new(FSRByteInfo::new(&const_map.lines, expr.get_meta().clone())),
                    });
                } else {
                    panic!("not support this single op: {:?}", single_op);
                }
            }
            return (op_code);
        }

        op_code.append(&mut second);
        if let Some(s) = BytecodeOperator::get_op(
            expr.get_op(),
            Box::new(FSRByteInfo::new(&const_map.lines, expr.get_meta().clone())),
            attr_id,
        ) {
            op_code.push(s);
        } else {
            unimplemented!()
        }

        if let Some(single_op) = expr.get_single_op() {
            if single_op.eq(&SingleOp::Not) {
                op_code.push(BytecodeArg {
                    operator: BytecodeOperator::NotOperator,
                    arg: Box::new(ArgType::None),
                    info: Box::new(FSRByteInfo::new(&const_map.lines, expr.get_meta().clone())),
                });
            }
        }
        (op_code)
    }

    fn load_block(
        block: &FSRBlock,
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> (Vec<Vec<BytecodeArg>>) {
        let mut vs = vec![];
        for token in block.get_tokens() {
            let lines = Self::load_token_with_map(token, var_map, const_map, false, false);
            for line in lines {
                vs.push(line);
            }
        }

        if vs.is_empty() {
            vs.push(vec![BytecodeArg {
                operator: BytecodeOperator::Empty,
                arg: Box::new(ArgType::None),
                info: Box::new(FSRByteInfo::new(&const_map.lines, block.get_meta().clone())),
            }]);
        }

        (vs)
    }

    fn get_pos(meta: FSRPosition, lines: &[usize]) -> FSRPos {
        let offset = meta.get_offset();
        for line in lines.iter().enumerate() {
            let line_number = line.0;
            let line_offset = line.1;
            if offset >= *line_offset {
                return FSRPos {
                    line: line_number,
                    column: offset - *line_offset + 1,
                };
            }
        }

        panic!("Position not found for offset: {}", offset);
    }

    fn load_try_def(
        try_def: &'a FSRTryBlock,
        var_map: &'a mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> (Vec<Vec<BytecodeArg>>) {
        let mut vs = vec![];

        for token in try_def.get_block().get_tokens() {
            let lines = Self::load_token_with_map(token, var_map, const_map, false, false);
            for line in lines {
                vs.push(line);
            }
        }

        vs.push(vec![BytecodeArg {
            operator: BytecodeOperator::EndTry,
            arg: Box::new(ArgType::None),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                try_def.get_meta().clone(),
            )),
        }]);

        let catch_start = vs.len();

        for token in try_def.get_catch().body.get_tokens() {
            let lines = Self::load_token_with_map(token, var_map, const_map, false, false);
            for line in lines {
                vs.push(line);
            }
        }

        vs.insert(
            0,
            vec![BytecodeArg {
                operator: BytecodeOperator::Try,
                arg: Box::new(ArgType::TryCatch(
                    catch_start as u64 + 1,
                    vs.len() as u64 + 2,
                )),
                info: Box::new(FSRByteInfo::new(
                    &const_map.lines,
                    try_def.get_meta().clone(),
                )),
            }],
        );

        vs.push(vec![BytecodeArg {
            operator: BytecodeOperator::EndCatch,
            arg: Box::new(ArgType::None),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                try_def.get_meta().clone(),
            )),
        }]);

        (vs)
    }

    fn load_if_def(
        if_def: &FSRIf,
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> (Vec<Vec<BytecodeArg>>) {
        let test_exp = if_def.get_test();
        let mut vs = vec![];
        let mut v = Self::load_token_with_map(test_exp, var_map, const_map, false, false);
        let mut test_list = Vec::new();
        let mut t = v.remove(0);
        test_list.append(&mut t);

        let block_items = Self::load_block(if_def.get_block(), var_map, const_map);
        let mut count_elses = 0;
        if let Some(s) = if_def.get_elses() {
            count_elses = s.get_elses().len();
        }
        test_list.push(BytecodeArg {
            operator: BytecodeOperator::IfTest,
            arg: Box::new(ArgType::IfTestNext((
                block_items.len() as u64,
                count_elses as u64,
            ))),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                if_def.get_meta().clone(),
            )),
        });
        vs.push(test_list);
        vs.extend(block_items);
        if let Some(s) = if_def.get_elses() {
            for e in s.get_elses() {
                let test_exp = e.get_test();

                let mut test_list = Vec::new();
                if let Some(t) = test_exp {
                    let block = e.get_block();
                    let block_items = Self::load_block(block, var_map, const_map);
                    test_list.push(BytecodeArg {
                        operator: BytecodeOperator::ElseIf,
                        arg: Box::new(ArgType::IfTestNext((block_items.len() as u64, 0))),
                        info: Box::new(FSRByteInfo::new(
                            &const_map.lines,
                            if_def.get_meta().clone(),
                        )),
                    });
                    let mut v = Self::load_token_with_map(t, var_map, const_map, false, false);
                    let mut t = v.remove(0);
                    test_list.append(&mut t);
                    test_list.push(BytecodeArg {
                        operator: BytecodeOperator::ElseIfTest,
                        arg: Box::new(ArgType::IfTestNext((block_items.len() as u64, 0))),
                        info: Box::new(FSRByteInfo::new(
                            &const_map.lines,
                            if_def.get_meta().clone(),
                        )),
                    });
                    vs.push(test_list);
                    vs.extend(block_items);
                } else {
                    let block = e.get_block();
                    let block_items = Self::load_block(block, var_map, const_map);
                    test_list.push(BytecodeArg {
                        operator: BytecodeOperator::Else,
                        arg: Box::new(ArgType::IfTestNext((block_items.len() as u64, 0))),
                        info: Box::new(FSRByteInfo::new(
                            &const_map.lines,
                            if_def.get_meta().clone(),
                        )),
                    });

                    vs.push(test_list);
                    vs.extend(block_items);
                }
            }
        }

        let end_if = vec![BytecodeArg {
            operator: BytecodeOperator::IfBlockEnd,
            arg: Box::new(ArgType::None),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                if_def.get_meta().clone(),
            )),
        }];
        vs.push(end_if);
        (vs)
    }

    #[allow(unused)]
    fn load_for_def(
        for_def: &FSRFor,
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> (Vec<Vec<BytecodeArg>>) {
        let mut result = vec![];

        let v = Self::load_token_with_map(for_def.get_expr(), var_map, const_map, false, false);
        let mut expr = v;

        let mut t = expr.remove(0);
        if !var_map.last_mut().unwrap().has_attr("__iter__") {
            var_map.last_mut().unwrap().insert_attr("__iter__");
        }
        let id = var_map.last_mut().unwrap().get_attr("__iter__").unwrap();
        t.push(BytecodeArg {
            operator: BytecodeOperator::ForBlockRefAdd,
            arg: Box::new(ArgType::None),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                for_def.get_meta().clone(),
            )),
        });

        let mut block_items = Self::load_block(for_def.get_block(), var_map, const_map);
        t.push(BytecodeArg {
            operator: BytecodeOperator::LoadForIter,
            arg: Box::new(ArgType::ForLine(block_items.len() as u64 + 3)),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                for_def.get_meta().clone(),
            )),
        });
        result.push(t);

        let mut load_next = Vec::new();

        let arg_id = ensure_var_id!(var_map, for_def.get_var_name());

        load_next.push(BytecodeArg {
            operator: BytecodeOperator::SpecialLoadFor,
            arg: Box::new(ArgType::Local(LocalVar::new(
                arg_id,
                for_def.get_var_name().to_string(),
                false,
                None,
            ))),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                for_def.get_meta().clone(),
            )),
        });

        result.push(load_next);
        result.append(&mut block_items);
        let end = vec![BytecodeArg {
            operator: BytecodeOperator::ForBlockEnd,
            arg: Box::new(ArgType::ForEnd(result.len() as i64 - 1)),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                for_def.get_meta().clone(),
            )),
        }];

        result.push(end);
        (result)
    }

    fn load_while_def(
        while_def: &FSRWhile,
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> (Vec<Vec<BytecodeArg>>) {
        let test_exp = while_def.get_test();
        let mut vs = vec![];
        let mut v = Self::load_token_with_map(test_exp, var_map, const_map, false, false);
        let mut test_list = Vec::new();
        let mut t = v.remove(0);
        test_list.append(&mut t);

        let block_items = Self::load_block(while_def.get_block(), var_map, const_map);
        test_list.push(BytecodeArg {
            operator: BytecodeOperator::WhileTest,
            arg: Box::new(ArgType::WhileTest(block_items.len() as u64 + 1)),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                while_def.get_meta().clone(),
            )),
        });
        vs.push(test_list);
        let len = block_items.len();
        //let l = block_items.0.get_mut(len - 1).unwrap();
        let end = BytecodeArg {
            operator: BytecodeOperator::WhileBlockEnd,
            arg: Box::new(ArgType::WhileEnd(len as i64 + 1)),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                while_def.get_meta().clone(),
            )),
        };
        vs.extend(block_items);
        vs.push(vec![end]);
        (vs)
    }

    fn load_break(info: Box<FSRByteInfo>) -> Vec<BytecodeArg> {
        let break_list = vec![BytecodeArg {
            operator: BytecodeOperator::Break,
            arg: Box::new(ArgType::None),
            info,
        }];
        break_list
    }

    fn load_continue(info: Box<FSRByteInfo>) -> Vec<BytecodeArg> {
        let continue_list = vec![BytecodeArg {
            operator: BytecodeOperator::Continue,
            arg: Box::new(ArgType::None),
            info,
        }];
        continue_list
    }

    fn load_import(
        import: &'a FSRImport,
        var_map: &'a mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> (Vec<Vec<BytecodeArg>>) {
        let name = import.module_name.last().unwrap();
        let id = ensure_var_id!(var_map, name);
        let import_list = vec![BytecodeArg {
            operator: BytecodeOperator::Import,
            arg: Box::new(ArgType::ImportModule(
                id,
                import.module_name.iter().map(|x| x.to_string()).collect(),
            )),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                import.get_meta().clone(),
            )),
        }];

        (vec![import_list])
    }

    // iter stack and join with ::, like __main__::fn_name
    // fn get_cur_name(map: &Vec<VarMap>, name: &str) -> String {
    //     let mut res_vec = vec![];
    //     for v in map.iter().skip(1) {
    //         res_vec.push(v.name.as_str());
    //     }
    //     res_vec.push(name);
    //     res_vec.join("::")
    // }

    fn load_token_with_map(
        token: &FSRToken,
        var_map: &mut Vec<VarMap>,
        byte_context: &mut BytecodeContext,
        is_attr: bool,
        is_method_call: bool,
    ) -> (Vec<Vec<BytecodeArg>>) {
        if let FSRToken::Expr(expr) = token {
            let v = Self::load_expr(expr, var_map, byte_context);
            return (vec![v]);
        } else if let FSRToken::Variable(v) = token {
            let v = Self::load_variable(v, var_map, false, byte_context);
            match v {
                AttrIdOrCode::Bytecode(bytecode_args) => return vec![bytecode_args],
                AttrIdOrCode::AttrId(_) => todo!(),
            }
        } else if let FSRToken::Module(m) = token {
            let mut vs = vec![];
            let ref_self = var_map;
            for token in &m.tokens {
                let lines = Self::load_token_with_map(token, ref_self, byte_context, false, false);
                for line in lines {
                    vs.push(line);
                }
            }

            return vs;
        } else if let FSRToken::IfExp(if_def) = token {
            let v = Self::load_if_def(if_def, var_map, byte_context);

            return v;
        } else if let FSRToken::Assign(assign) = token {
            let v = Self::load_assign(assign, var_map, byte_context);
            return vec![v];
        } else if let FSRToken::WhileExp(while_def) = token {
            let v = Self::load_while_def(while_def, var_map, byte_context);
            return v;
        } else if let FSRToken::Block(block) = token {
            let v = Self::load_block(block, var_map, byte_context);
            return v;
        } else if let FSRToken::Call(call) = token {
            let v = Self::load_call(call, var_map, is_attr, is_method_call, byte_context);
            return vec![v];
        } else if let FSRToken::Getter(getter) = token {
            let v = Self::load_list_getter(
                getter,
                var_map,
                is_attr,
                is_method_call,
                false,
                byte_context,
            );
            return (vec![v]);
        } else if let FSRToken::Constant(c) = token {
            let v = Self::load_constant(c, var_map, byte_context);
            return (vec![v]);
        } else if let FSRToken::FunctionDef(fn_def) = token {
            if fn_def.is_lambda() {
                let v = Self::load_function(fn_def, var_map, byte_context);
                //let fn_name = Self::get_cur_name(var_map, fn_def.get_name());
                //const_map.fn_def_map.insert(fn_name.clone(), v);
                var_map.last_mut().unwrap().sub_fn_def.push(Bytecode {
                    name: fn_def.get_name().to_string(),
                    context: BytecodeContext::new(vec![]),
                    bytecode: v.0,
                    var_map: v.1,
                    is_jit: false,
                    is_async: false,
                });
                let c_id = var_map
                    .last_mut()
                    .unwrap()
                    .get_var(fn_def.get_name())
                    .cloned()
                    .unwrap();
                let result = vec![BytecodeArg {
                    operator: BytecodeOperator::Load,
                    arg: Box::new(ArgType::Local(LocalVar::new(
                        c_id,
                        fn_def.get_name().to_string(),
                        false,
                        None,
                    ))),
                    info: Box::new(FSRByteInfo::new(
                        &byte_context.lines,
                        fn_def.get_meta().clone(),
                    )),
                }];
                return (vec![result]);
            }

            let v = Self::load_function(fn_def, var_map, byte_context);
            return v.0;
        } else if let FSRToken::Class(cls) = token {
            let v = Self::load_class(cls, var_map, byte_context);
            return v;
        } else if let FSRToken::Return(ret) = token {
            let v = Self::load_ret(ret, var_map, byte_context);
            return vec![v];
        } else if let FSRToken::List(list) = token {
            let v = Self::load_list(list, var_map, byte_context);
            return vec![v];
        } else if let FSRToken::Break(b) = token {
            let v = Self::load_break(Box::new(FSRByteInfo::new(&byte_context.lines, b.clone())));
            return vec![v];
        } else if let FSRToken::Continue(c) = token {
            let v = Self::load_continue(Box::new(FSRByteInfo::new(&byte_context.lines, c.clone())));
            return vec![v];
        } else if let FSRToken::ForBlock(b) = token {
            let v = Self::load_for_def(b, var_map, byte_context);
            return (v);
        } else if let FSRToken::Import(import) = token {
            let v = Self::load_import(import, var_map, byte_context);
            return v;
        } else if let FSRToken::StackExpr(st) = token {
            let v = Self::load_stack_expr(st, var_map, byte_context, is_attr, is_method_call);
            return vec![v];
        } else if let FSRToken::TryBlock(try_block) = token {
            let v = Self::load_try_def(try_block, var_map, byte_context);
            return v;
        } else if let FSRToken::EmptyExpr(_) = token {
            return vec![];
        }

        unimplemented!()
    }

    fn load_dot_assign(
        token: &FSRAssign,
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> Option<(Vec<BytecodeArg>)> {
        let mut result_list = Vec::new();
        if let FSRToken::Expr(v) = &**token.get_left() {
            if !v.get_op().eq(".") {
                return None;
            }
            let attr_name = if let FSRToken::Variable(attr_name) = v.get_right() {
                attr_name.get_name()
            } else {
                return None;
            };

            let attr_id = {
                if !var_map.last_mut().unwrap().has_attr(attr_name) {
                    var_map.last_mut().unwrap().insert_attr(attr_name);
                }

                let attr_id = var_map.last_mut().unwrap().get_attr(attr_name).unwrap();
                *attr_id
            };

            let mut left =
                Self::load_token_with_map(v.get_left(), var_map, const_map, false, false);

            let mut right = Self::load_token_with_map(
                token.get_assign_expr(),
                var_map,
                const_map,
                false,
                false,
            );

            result_list.append(&mut right[0]);
            result_list.append(&mut left[0]);
            //right.1.last_mut().unwrap().insert_var(v.get_name());
            //let id = right.1.last_mut().unwrap().get_var(v.get_name()).unwrap();
            result_list.push(BytecodeArg {
                operator: BytecodeOperator::AssignAttr,
                arg: Box::new(ArgType::Attr(
                    attr_id,
                    attr_name.to_string(),
                    OpAssign::from_str(&token.op_assign),
                )),
                info: Box::new(FSRByteInfo::new(&const_map.lines, v.get_meta().clone())),
            });
            return Some(result_list);
        }
        None
    }

    fn load_getter_assign(
        token: &FSRAssign,
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> Option<(Vec<BytecodeArg>)> {
        let mut result_list = Vec::new();
        if let FSRToken::Getter(v) = &**token.get_left() {
            let mut left = Self::load_list_getter(v, var_map, false, false, true, const_map);

            let mut right = Self::load_token_with_map(
                token.get_assign_expr(),
                var_map,
                const_map,
                false,
                false,
            );

            result_list.append(&mut right[0]);
            result_list.append(&mut left);
            //right.1.last_mut().unwrap().insert_var(v.get_name());
            //let id = right.1.last_mut().unwrap().get_var(v.get_name()).unwrap();
            result_list.push(BytecodeArg {
                operator: BytecodeOperator::AssignContainer,
                arg: Box::new(ArgType::None),
                info: Box::new(FSRByteInfo::new(&const_map.lines, v.get_meta().clone())),
            });
            return Some(result_list);
        }
        None
    }

    fn load_assign_helper(
        token: &FSRAssign,
        v: &FSRVariable,
        var_map: &mut Vec<VarMap>,
        bc_map: &mut BytecodeContext,
        result_list: &mut Vec<BytecodeArg>,
        type_id: Option<FSRSType>,
    ) {
        let mut right =
            Self::load_token_with_map(token.get_assign_expr(), var_map, bc_map, false, false);
        result_list.append(&mut right[0]);
        var_map.last_mut().unwrap().insert_var(v.get_name());
        let id = var_map.last_mut().unwrap().get_var(v.get_name()).unwrap();
        if let Some(ref_map) = bc_map.ref_map_stack.last() {
            if ref_map.get(v.get_name()).copied().unwrap_or(false) {
                result_list.push(BytecodeArg {
                    operator: BytecodeOperator::Assign,
                    arg: Box::new(ArgType::ClosureVar((
                        *id,
                        v.get_name().to_string(),
                        OpAssign::from_str(&token.op_assign),
                    ))),
                    info: Box::new(FSRByteInfo::new(&bc_map.lines, v.get_meta().clone())),
                });
                //return (result_list);
            }
        }
        let op_assign = OpAssign::from_str(&token.op_assign);
        let mut local_var = LocalVar::new(
                *id,
                v.get_name().to_string(),
                false,
                op_assign,
            );

        local_var.var_type = type_id;
        result_list.push(BytecodeArg {
            operator: BytecodeOperator::Assign,
            arg: Box::new(ArgType::Local(local_var)),
            info: Box::new(FSRByteInfo::new(&bc_map.lines, v.get_meta().clone())),
        });
    }

    fn load_assign(
        token: &FSRAssign,
        var_map: &mut Vec<VarMap>,
        bc_map: &mut BytecodeContext,
    ) -> Vec<BytecodeArg> {
        let mut result_list = Vec::new();
        if let FSRToken::Variable(v) = &**token.get_left() {
            if bc_map.is_static {
                if let Some(type_hint) = v.get_type_hint() {
                    // .....Wait to implement
                    let type_name = type_hint.name.as_str();
                    let type_id = bc_map
                        .type_info
                        .get_type(type_name)
                        .expect("wait to impl: if not getting type_id");
                    Self::load_assign_helper(
                        token,
                        v,
                        var_map,
                        bc_map,
                        &mut result_list,
                        Some(type_id),
                    );
                    return result_list;
                } else {
                    panic!("Static variable {} must have type hint", v.get_name());
                }
            }

            Self::load_assign_helper(token, v, var_map, bc_map, &mut result_list, None);
            (result_list)
        } else if let Some(v) = Self::load_getter_assign(token, var_map, bc_map) {
            v
        } else if let Some(v) = Self::load_dot_assign(token, var_map, bc_map) {
            v
        } else {
            let mut left =
                Self::load_token_with_map(token.get_left(), var_map, bc_map, false, false);
            let mut right =
                Self::load_token_with_map(token.get_assign_expr(), var_map, bc_map, false, false);
            result_list.append(&mut right[0]);
            result_list.append(&mut left[0]);
            result_list.push(BytecodeArg {
                operator: BytecodeOperator::Assign,
                arg: Box::new(ArgType::None),
                info: Box::new(FSRByteInfo::new(&bc_map.lines, token.get_meta().clone())),
            });
            result_list
        }
    }

    fn load_constant(
        token: &FSRConstant,
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> Vec<BytecodeArg> {
        if let FSRConstType::FormatString(format_string) = token.get_const_type() {
            let mut result = vec![];
            let args_len = format_string.arg_expr.len();
            for token in format_string.arg_expr.iter().rev() {
                let mut v =
                    Bytecode::load_token_with_map(&token.expr, var_map, const_map, false, false);
                if v.is_empty() {
                    continue;
                }
                if v.len() > 1 {
                    panic!("Format string argument should be a single expression");
                }

                result.append(&mut v[0]);
            }

            result.push(BytecodeArg {
                operator: BytecodeOperator::FormatString,
                arg: Box::new(ArgType::FormatStringLen(
                    args_len as u64,
                    format_string.format_str.clone(),
                )),
                info: Box::new(FSRByteInfo::new(&const_map.lines, token.get_meta().clone())),
            });

            return result;
        }

        let c = token.get_const_str();

        let id = ensure_const_id!(var_map, c);

        let mut result_list = vec![BytecodeArg {
            operator: BytecodeOperator::Load,
            arg: Box::new(ArgType::Const(id)),
            info: Box::new(FSRByteInfo::new(&const_map.lines, token.get_meta().clone())),
        }];

        (result_list)
    }

    fn load_list(
        token: &FSRListFrontEnd,
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> (Vec<BytecodeArg>) {
        let mut result_list = Vec::new();
        for sub_t in token.get_items().iter().rev() {
            let v = Bytecode::load_token_with_map(sub_t, var_map, const_map, false, false);
            let mut expr = v;
            if !expr.is_empty() {
                result_list.append(&mut expr[0]);
            } else {
                result_list.push(BytecodeArg {
                    operator: BytecodeOperator::LoadList,
                    arg: Box::new(ArgType::LoadListNumber(0)),
                    info: Box::new(FSRByteInfo::new(&const_map.lines, sub_t.get_meta().clone())),
                });
                return (result_list);
            }
        }

        let load_list = BytecodeArg {
            operator: BytecodeOperator::LoadList,
            arg: Box::new(ArgType::LoadListNumber(token.get_items().len())),
            info: Box::new(FSRByteInfo::new(&const_map.lines, token.get_meta().clone())),
        };
        result_list.push(load_list);
        (result_list)
    }

    fn load_ret(
        ret: &FSRReturn,
        var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> (Vec<BytecodeArg>) {
        let v = Self::load_token_with_map(ret.get_return_expr(), var_map, const_map, false, false);
        let mut ret_expr = Vec::new();
        let mut r = v;
        if !r.is_empty() {
            ret_expr.append(&mut r[0]);
        }
        ret_expr.push(BytecodeArg {
            operator: BytecodeOperator::ReturnValue,
            arg: Box::new(ArgType::None),
            info: Box::new(FSRByteInfo::new(&const_map.lines, ret.get_meta().clone())),
        });

        (ret_expr)
    }

    fn process_integer(ps: &str) -> i64 {
        //let new_ps = ps.replace("_", "");
        let s = ps;
        let (s, base) = if s.starts_with("0x") || s.starts_with("0X") {
            (&s[2..], 16)
        } else if s.starts_with("0b") || s.starts_with("0B") {
            (&s[2..], 2)
        } else if s.starts_with("0o") || s.starts_with("0O") {
            (&s[2..], 8)
        } else {
            (s, 10)
        };
        match i64::from_str_radix(s, base) {
            Ok(value) => value,
            Err(_) => {
                panic!("Invalid integer literal: {}", ps);
            }
        }
    }

    fn load_function(
        fn_def: &FSRFnDef,
        var_map: &mut Vec<VarMap>,
        bytecontext: &mut BytecodeContext,
    ) -> (Vec<Vec<BytecodeArg>>, VarMap) {
        let mut result = vec![];
        let name = fn_def.get_name();
        let arg_id = ensure_var_id!(var_map, name);
        let store_to_cell = if let Some(ref_map) = bytecontext.ref_map_stack.last() {
            ref_map.get(name).copied().unwrap_or(false)
                && bytecontext.contains_variable_in_ref_stack(name)
        } else {
            false
        };
        let mut load_args = Vec::new();

        let fn_var_map = VarMap::new(fn_def.get_name());
        var_map.push(fn_var_map);
        let mut hash_map_ref_map = HashMap::new();
        for arg in fn_def.ref_map.borrow().iter() {
            hash_map_ref_map.insert(arg.0.to_string(), arg.1.is_defined);
        }
        bytecontext.ref_map_stack.push(hash_map_ref_map);
        let args: &Vec<FSRToken> = fn_def.get_args();
        for arg in args {
            if let FSRToken::Variable(v) = arg {
                let mut a = Self::load_assign_arg(v, var_map, bytecontext);
                load_args.append(&mut a);
            }
        }

        //let mut fn_var_map_ref = &mut fn_var_map;

        let mut define_fn = Vec::new();
        let mut arg_len = 0;
        let mut args_save = vec![];
        for arg in args {
            if let FSRToken::Variable(v) = arg {
                args_save.push(v.get_name().to_string());
                arg_len += 1;
            }
        }

        args_save.reverse();

        let body = fn_def.get_body();
        bytecontext.cur_fn_name.push(name.to_string());
        let cur_name = bytecontext.cur_fn_name.join("::").to_string();
        let origin_is_static = bytecontext.is_static;
        bytecontext.is_static = fn_def.is_static();
        let v = Self::load_block(body, var_map, bytecontext);
        bytecontext.is_static = origin_is_static;
        bytecontext.cur_fn_name.pop();
        let mut fn_body = v;

        let v = var_map.pop().unwrap();

        let mut const_map = &v.const_map;
        let mut const_loader = vec![];
        for const_var in const_map {
            match const_var.0 {
                FSROrinStr2::Integer(i, v) => {
                    const_loader.push(BytecodeArg {
                        operator: BytecodeOperator::LoadConst,
                        arg: Box::new(ArgType::ConstInteger(
                            *const_var.1,
                            Self::process_integer(i.as_str()),
                            *v,
                        )),
                        info: Box::new(FSRByteInfo::new(&bytecontext.lines, FSRPosition::new())),
                    });
                }
                FSROrinStr2::Float(f, v) => {
                    const_loader.push(BytecodeArg {
                        operator: BytecodeOperator::LoadConst,
                        arg: Box::new(ArgType::ConstFloat(*const_var.1, f.to_string(), *v)),
                        info: Box::new(FSRByteInfo::new(&bytecontext.lines, FSRPosition::new())),
                    });
                }
                FSROrinStr2::String(s) => {
                    const_loader.push(BytecodeArg {
                        operator: BytecodeOperator::LoadConst,
                        arg: Box::new(ArgType::ConstString(*const_var.1, s.to_string())),
                        info: Box::new(FSRByteInfo::new(&bytecontext.lines, FSRPosition::new())),
                    });
                }
            }
        }

        fn_body.insert(0, const_loader);

        for sub_def in v.sub_fn_def.iter() {
            fn_body.splice(0..0, sub_def.bytecode.clone());
        }

        define_fn.push(BytecodeArg {
            operator: BytecodeOperator::DefineFn,
            arg: Box::new(ArgType::DefineFnArgs(
                arg_id,
                name.to_string(),
                cur_name.to_string(),
                args_save,
                store_to_cell,
            )),
            info: Box::new(FSRByteInfo::new(
                &bytecontext.lines,
                fn_def.get_meta().clone(),
            )),
        });

        fn_body.insert(0, load_args);
        if let Some(last) = fn_body.last() {
            if last.last().is_none()
                || last.last().unwrap().operator != BytecodeOperator::ReturnValue
            {
                fn_body.push(vec![BytecodeArg {
                    operator: BytecodeOperator::ReturnValue,
                    arg: Box::new(ArgType::None),
                    info: Box::new(FSRByteInfo::new(
                        &bytecontext.lines,
                        fn_def.get_meta().clone(),
                    )),
                }]);
            }
        }

        let mut var_map = VarMap::new("_");
        var_map.attr_id = AtomicU64::new(v.attr_id.load(Ordering::Relaxed));
        var_map.var_id = AtomicU64::new(v.var_id.load(Ordering::Relaxed));
        var_map.var_map = v.var_map.clone();
        var_map.attr_map = v.attr_map.clone();
        var_map.const_map = v.const_map.clone();
        let fn_def = FnDef {
            code: fn_body.clone(),
            var_map,
            is_jit: fn_def.is_jit(),
            is_async: fn_def.is_async(),
            is_static: fn_def.is_static(),
        };
        bytecontext.fn_def_map.insert(cur_name, fn_def);

        result.push(define_fn);
        // result.push(load_args);

        //result.push(define_fn);
        if !fn_body.is_empty() {
            //result.extend(fn_body);
        }

        bytecontext.ref_map_stack.pop();

        //result.push(end_of_fn);

        // result.push(end_list);
        (result, v)
    }

    fn load_class(
        class_def: &FSRClassFrontEnd,
        mut var_map: &mut Vec<VarMap>,
        const_map: &mut BytecodeContext,
    ) -> (Vec<Vec<BytecodeArg>>) {
        let mut result = vec![];
        let name = class_def.get_name();
        let arg_id = ensure_var_id!(var_map, name);
        let store_to_cell = if let Some(ref_map) = const_map.ref_map_stack.last() {
            ref_map.get(name).cloned().unwrap_or(false)
                && const_map.contains_variable_in_ref_stack(name)
        } else {
            false
        };

        let class_var_map = VarMap::new(class_def.get_name());
        var_map.push(class_var_map);
        const_map.cur_fn_name.push(name.to_string());
        let cur_name = const_map.cur_fn_name.join("::").to_string();
        let mut v = Self::load_block(class_def.get_block(), var_map, const_map);
        const_map.cur_fn_name.pop();
        let last = var_map.last().unwrap();

        let ans = vec![
            // op_arg,
            BytecodeArg {
                operator: BytecodeOperator::ClassDef,
                arg: Box::new(ArgType::Local(LocalVar::new(
                    arg_id,
                    name.to_string(),
                    store_to_cell,
                    None,
                ))),
                info: Box::new(FSRByteInfo::new(
                    &const_map.lines,
                    class_def.get_meta().clone(),
                )),
            },
        ];

        result.push(ans);
        result.extend(v);
        let end_of_cls = vec![BytecodeArg {
            operator: BytecodeOperator::EndDefineClass,
            arg: Box::new(ArgType::Local(LocalVar::new(
                arg_id,
                name.to_string(),
                false,
                None,
            ))),
            info: Box::new(FSRByteInfo::new(
                &const_map.lines,
                class_def.get_meta().clone(),
            )),
        }];
        result.push(end_of_cls);
        var_map.pop();
        (result)
    }

    fn load_isolate_block(
        token: &FSRToken,
        const_map: &mut BytecodeContext,
    ) -> (Vec<Vec<BytecodeArg>>, VarMap) {
        let mut var_map = vec![VarMap::new("__main__")];
        if let FSRToken::Module(m) = token {
            let mut hash_map_ref_map = HashMap::new();
            for arg in m.ref_map.borrow().iter() {
                hash_map_ref_map.insert(arg.0.to_string(), arg.1.is_defined);
            }
            const_map.ref_map_stack.push(hash_map_ref_map);
        }

        let mut v = Self::load_token_with_map(token, &mut var_map, const_map, false, false);
        let var = var_map.pop().unwrap();
        for sub_def in var.sub_fn_def.iter() {
            v.splice(0..0, sub_def.bytecode.clone());
        }

        (v, var)
    }

    pub fn load_ast(_name: &str, token: FSRToken, lines: Vec<usize>) -> HashMap<String, Bytecode> {
        let mut const_table = BytecodeContext::new(lines);
        let vs = Self::load_isolate_block(&token, &mut const_table);
        let mut result = vec![];
        for v in vs.0 {
            let single_line = Vec::from_iter(v);
            result.push(single_line);
        }

        let mut const_map = &vs.1.const_map;
        let mut const_loader = vec![];
        for const_var in const_map {
            match const_var.0 {
                FSROrinStr2::Integer(i, v) => {
                    const_loader.push(BytecodeArg {
                        operator: BytecodeOperator::LoadConst,
                        arg: Box::new(ArgType::ConstInteger(
                            *const_var.1,
                            Self::process_integer(i.as_str()),
                            *v,
                        )),
                        info: Box::new(FSRByteInfo::new(&const_table.lines, FSRPosition::new())),
                    });
                }
                FSROrinStr2::Float(f, v) => {
                    const_loader.push(BytecodeArg {
                        operator: BytecodeOperator::LoadConst,
                        arg: Box::new(ArgType::ConstFloat(*const_var.1, f.to_string(), *v)),
                        info: Box::new(FSRByteInfo::new(&const_table.lines, FSRPosition::new())),
                    });
                }
                FSROrinStr2::String(s) => {
                    const_loader.push(BytecodeArg {
                        operator: BytecodeOperator::LoadConst,
                        arg: Box::new(ArgType::ConstString(*const_var.1, s.to_string())),
                        info: Box::new(FSRByteInfo::new(&const_table.lines, FSRPosition::new())),
                    });
                }
            }
        }

        result.insert(0, const_loader);

        let mut res = HashMap::new();
        res.insert(
            "__main__".to_string(),
            Bytecode {
                name: "__main__".to_string(),
                context: BytecodeContext::new(vec![]),
                bytecode: result,
                var_map: vs.1,
                is_jit: false,
                is_async: false,
            },
        );

        let codes = const_table.fn_def_map;

        for code in codes {
            let bytecode = Bytecode {
                name: code.0.to_string(),
                context: BytecodeContext::new(vec![]),
                bytecode: code.1.code,
                var_map: code.1.var_map,
                is_jit: code.1.is_jit,
                is_async: code.1.is_async,
            };

            res.insert(code.0.to_string(), bytecode);
        }

        res
    }

    pub fn compile(name: &str, code: &str) -> HashMap<String, Bytecode> {
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(code.as_bytes(), meta).unwrap();
        Self::load_ast(name, FSRToken::Module(token.0), token.1)
    }
}

#[allow(unused)]
mod test {
    use crate::{
        backend::compiler::bytecode::Bytecode,
        frontend::ast::token::{
            base::{FSRPosition, FSRToken},
            module::FSRModuleFrontEnd,
        },
    };

    #[test]
    fn test_1() {
        let expr = "
        b[abc()]
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_stack_expr() {
        let expr = "
a.abc[0]
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_method_call() {
        let expr = "
a.abc(0)
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_class_getter() {
        let expr = "
        Test::abc()
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_try_block() {
        let expr = "
        try {
            println(1)
        } catch {
            println(2)
        }
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn closure_test() {
        let expr = "
        fn abc() {
            a = 1
            b = 1
            fn ddc() {
                return a
            }
        }
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn lambda_test() {
        let expr = "a = |a, b| { a + b }";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_class() {
        let expr = "
        class Ddc {
            fn __new__(self, ddc) {
                self.ddc = 123 + 1
            }
        }
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn lambda_closure_test() {
        let expr = "
        fn abc3() {
            a = 1
            fn ddc() {
                a = a + 1
                println(a)
                return a
            }
            a = 2
            fn abcd() {
                return ddc
            }

            return abcd()
        }
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn lambda_closure_test2() {
        let expr = "
        fn abc() {
            fn fib(n) {
                if n == 1 or n == 2 {
                    return 1
                } else {
                    return fib(n - 1) + fib(n - 2)
                }
            }
            result = fib(30)
            println(result)

            gc_info()
        }

        abc()
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_list_getter() {
        let expr = "
        a = [1, 2, 3]
        println(a[0])

        b = [[1,2,3]]
        c = b[0][0]
        println(c)";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_c() {
        let expr = "b = 10 + -1 * 10";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_assign() {
        let expr = "a.c = 1
a[0] = 1
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn optimize_for() {
        let expr = "
        for i in 0..3000000 {
            
        }
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_not_attr() {
        let expr = "
        not a.contains(0) && abc
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_simple() {
        let expr = "
fn abc() {
    gc_info()
}

abc()
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_getter_assign() {
        let expr = "
        a = [1, 2]
        a[1 + 1] = 1
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_dot_assign() {
        let expr = "
        a.c = 1
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_method_call_or_not() {
        let expr = "
        t.index = t.index + t.abc()
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_import() {
        let expr = "thread::Thread::thread_id
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_is_jit() {
        let expr = "
        @jit
        fn abc() {
            a = 1
            b = 1
        }
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_for() {
        let expr = "
            for i in a {
                a = 1
            }
        
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_logic() {
        let expr = "
        a or test()
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_assign_2() {
        let expr = "
        b = 10 + -1 * 10
        println(b)
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_class_getter2() {
        let expr = "
        f = fs::File.open(\"./.gitignore\")
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_exp_throw() {
        let expr = "
        a.try
        ";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_format_string() {
        let expr = r#"f"Hello, {name}""#;

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }
}
