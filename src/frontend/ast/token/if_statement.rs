use super::base::FSRToken;
use super::statement::ASTTokenInterface;
use super::statement::ASTTokenEnum;



pub struct FSRIf<'a> {
    test    : Box<FSRToken<'a>>,
    body    : Vec<FSRToken<'a>>
}

impl ASTTokenInterface for FSRIf<'_> {
    fn get_expect_states() -> Vec<ASTTokenEnum> {
        unimplemented!()
    }
}