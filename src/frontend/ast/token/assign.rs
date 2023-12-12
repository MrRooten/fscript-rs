use super::base::FSRToken;

pub struct FSRAssign<'a> {
    targets     : Vec<FSRToken<'a>>,
    value       : Vec<FSRToken<'a>>   
}