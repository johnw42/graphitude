use std::{collections::HashMap, marker::PhantomData};

use crate::id::NodeId;

pub trait Directedness {
    type NodeData;
    type EdgeData;
}

struct Directed<N, E>(PhantomData<(N, E)>);

impl<N, E> Directedness for Directed<N, E> {
    type NodeData = N;
    type EdgeData = E;
}

struct Undirected<N, E>(PhantomData<(N, E)>);

impl<N, E> Directedness for Undirected<N, E> {
    type NodeData = N;
    type EdgeData = E;
}

pub trait EdgeStorage {
    type Adjacency;
    type Neighbors<N, E, T>;
}

struct NeighborList;

impl EdgeStorage for NeighborList {
    type Adjacency = ();
    type Neighbors<N, E, T> = HashMap<NodeId<N, E>, T>;
}
