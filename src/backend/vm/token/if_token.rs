use super::state::{ASTTokenEnum, ASTTokenInterface};

pub struct FSRIfToken {

}

impl ASTTokenInterface for FSRIfToken {
    fn get_expect_states() -> Vec<ASTTokenEnum> {
        unimplemented!()
    }
}