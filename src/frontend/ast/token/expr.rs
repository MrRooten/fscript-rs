use super::base::FSRToken;

pub struct FSRExpr<'a> {
    value       : Box<FSRToken<'a>>
}