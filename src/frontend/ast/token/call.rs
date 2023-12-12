use super::base::FSRToken;

pub struct FSRCall<'a> {
    name        : &'a str,
    args        : Vec<FSRToken<'a>>
}