use std::{sync::Arc, time::Duration};

use kube::runtime::controller::Action;

pub fn default_error_policy<E, D>(_error: &E, _ctx: Arc<D>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
