use crate::backend::types::asynclib::future::FSRFuture;

pub struct FSRScheduler {
    futures: Vec<FSRFuture>,
}
