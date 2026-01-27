use std::{collections::HashMap, marker::PhantomData};

use crate::id::NodeId;

pub trait Directedness {
    type NodeData;
    type EdgeData;
}

struct Directed<V, E>(PhantomData<(V, E)>);

impl<V, E> Directedness for Directed<V, E> {
    type NodeData = V;
    type EdgeData = E;
}

struct Undirected<V, E>(PhantomData<(V, E)>);

impl<V, E> Directedness for Undirected<V, E> {
    type NodeData = V;
    type EdgeData = E;
}

pub trait EdgeStorage {
    type Adjacency;
    type Neighbors<V, E, T>;
}

struct NeighborList;

impl EdgeStorage for NeighborList {
    type Adjacency = ();
    type Neighbors<V, E, T> = HashMap<NodeId<V, E>, T>;
}
