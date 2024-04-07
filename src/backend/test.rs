#[cfg(test)]
mod tests {
    use crate::{backend::compiler::bytecode::Bytecode, frontend::ast::token::{base::{FSRMeta, FSRToken}, expr::FSRExpr, module::FSRModuleFrontEnd}};

    #[test]
    fn test_1() {
        let expr = "
        b(abc) + a.a + b * c + d
        a + b
        ";
        let meta = FSRMeta::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast(FSRToken::Module(token));
        println!("{:#?}", v);
    }
}