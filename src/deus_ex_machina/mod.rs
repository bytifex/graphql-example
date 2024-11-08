use std::sync::Arc;

use async_graphql::{
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextResolve, ResolveInfo},
    ServerResult, Value,
};

use crate::state::State;

struct DeusExMachinaInner {
    state: State,
}

impl DeusExMachinaInner {
    pub fn new(state: State) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl Extension for DeusExMachinaInner {
    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        log::trace!(
            "resolving field, path = {:?}",
            info.path_node.to_string_vec()
        );
        next.run(ctx, info).await
    }
}

pub struct DeusExMachina(Arc<DeusExMachinaInner>);

impl DeusExMachina {
    pub fn new(state: State) -> Self {
        Self(Arc::new(DeusExMachinaInner::new(state)))
    }
}

impl ExtensionFactory for DeusExMachina {
    fn create(&self) -> Arc<dyn Extension> {
        self.0.clone()
    }
}
