use super::statement::ASTTokenInterface;
use super::statement::ASTTokenEnum;


pub struct FSRIfToken {

}

impl ASTTokenInterface for FSRIfToken {
    fn get_expect_states() -> Vec<ASTTokenEnum> {
        unimplemented!()
    }
}