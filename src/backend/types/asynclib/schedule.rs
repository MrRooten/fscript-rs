use crate::backend::types::asynclib::future::FSRFuture;

pub struct FSRScheduler<'a> {
    futures: Vec<FSRFuture<'a>>,
}
