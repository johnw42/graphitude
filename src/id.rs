use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexId<V, E>(pub(crate) usize, PhantomData<(V, E)>);
