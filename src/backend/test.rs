#[cfg(test)]
mod tests {
    use crate::{backend::compiler::bytecode::Bytecode, frontend::ast::token::{base::FSRMeta, expr::FSRExpr}};

    #[test]
    fn test_1() {
        let expr = "a + b * c + d";
        let meta = FSRMeta::new();
        let token = FSRExpr::parse(expr.as_bytes(), false, meta).unwrap().0;
        let v = Bytecode::load_ast(token);
        println!("{:#?}", v);
    }
}