use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId<N, E>(pub(crate) usize, PhantomData<(N, E)>);
