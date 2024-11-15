use async_graphql::Subscription;
use futures_util::{Stream, StreamExt};

use crate::state::State;

pub struct Subscription {
    pub _state: State,
}

#[Subscription]
impl Subscription {
    pub async fn ticks(&self, seconds: u64) -> impl Stream<Item = usize> {
        let mut counter = 0;
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
            std::time::Duration::from_secs(seconds),
        ))
        .map(move |_| {
            counter += 1;
            counter
        })
    }
}
