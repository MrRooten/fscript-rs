#[derive(PartialEq, Clone)]
pub enum ASTState {
    WaitToken,
    StartToken,
    ContinueToken,
    TokenEnd,
}

pub enum ASTFunctionToken {
    FunctionName,
    FunctionArg,
}

pub enum ASTTokenEnum {
    KeyToken,
    VariableNameToken,
    ImportToken,
    FunctionToken,
    ClassToken,
    VariableAssign,
}

pub enum ASTStatement {
    FunctionDef,
    ClassDef,
    Assign,
    Return,
    ForState,
    WhileState,
    IfState,
    Continue,
}

pub struct ASTToken {}

impl ASTToken {
    pub fn new(token: ASTTokenEnum, value: &str) -> ASTToken {
        unimplemented!()
    }
}

pub trait ASTTokenInterface {
    fn get_expect_states() -> Vec<ASTTokenEnum>;
}

pub enum ASTImportState {}

pub enum ASTCodeState {}

pub enum ASTFunctionState {}

pub enum ASTClassState {}
