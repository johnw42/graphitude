#![allow(clippy::type_complexity)]

use std::collections::HashMap;

use crate::{
    AddEdgeResult, DirectednessTrait, EdgeIdTrait, EdgeMultiplicityTrait, Graph, GraphMut,
};

/// Utility for copying graphs with flexible transformations and mapping of node
/// and edge IDs.  Uses a builder pattern to allow configuring the copying
/// process with various options, such as directedness, edge multiplicity, and
/// transformation functions for node and edge data.  The copier can also track
/// the mapping of node and edge IDs from the source graph to the target graph
/// using user-provided maps.  This allows for complex copying scenarios, such
/// as copying between different graph types, applying transformations to the
/// data, and maintaining ID correspondences.
///
/// # Examples
///
/// To clone an existing graph with cloneable node and edge data:
/// ```ignore
/// let source_graph = ...; // Some graph with cloneable node and edge data
/// let target_graph = GraphCopier::new(&source_graph)
///     .clone_nodes()
///     .clone_edges()
///     .copy();
/// ```
/// To merge multiple existing graphs with transformations and ID mapping:
/// ```ignore
/// let source_graphs = ...; // Some Vec of graphs
/// let mut target_graph = ...; // Some existing graph to copy into
/// let mut node_id_map = HashMap::new();
/// let mut edge_id_map = HashMap::new();
/// for source_graph in source_graphs.iter() {
///     GraphCopier::new(source_graph)
///         .transform_nodes(|data| transform_node_data(data))
///         .transform_edges(|data| transform_edge_data(data))
///         .with_node_map(&mut node_id_map)
///         .with_edge_map(&mut edge_id_map)
///         .copy_into(&mut target_graph);
/// }
///
/// // Now `target_graph` has the copied structure and data, and
/// // `node_id_map` and `edge_id_map` contain the mappings of IDs
/// // from `source_graphs` to `target_graph`.
/// ```
pub struct GraphCopier<'g, G, D, M, NT, ET, NM, EM>
where
    G: Graph + ?Sized,
{
    /// The source graph to copy from.  This is a reference with lifetime `'g`
    /// to allow the copier to borrow data from the source graph during copying,
    /// such as for transformations or ID mappings.
    source: &'g G,
    /// The directedness to use for the target graph when creating a new graph with
    /// [`Self::copy`].  This is ignored when copying into an existing graph with
    /// [`Self::copy_into`], which will use the target graph's directedness instead.
    directedness: D,
    /// The edge multiplicity to use for the target graph when creating a new graph with
    /// [`Self::copy`].  This is ignored when copying into an existing graph with
    /// [`Self::copy_into`], which will use the target graph's edge multiplicity instead.
    edge_multiplicity: M,
    /// The node data transformation function to apply to each node's data
    /// during copying.  This is a function that takes a reference to the source
    /// graph's node data and produces the target graph's node data.
    node_transformer: NT,
    /// The edge data transformation function to apply to each edge's data
    /// during copying.  This is a function that takes a reference to the source
    /// graph's edge data and produces the target graph's edge data.
    edge_transformer: ET,
    /// An optional mutable reference to a map for tracking the mapping of node IDs
    /// from the source graph to the target graph.  If provided, this map will be
    /// cleared and populated during the copying process, allowing the caller to
    /// track how node IDs in the source graph correspond to node IDs in the target graph.
    node_map: NM,
    /// An optional mutable reference to a map for tracking the mapping of edge IDs
    /// from the source graph to the target graph.  If provided, this map will be
    /// cleared and populated during the copying process, allowing the caller to
    /// track how edge IDs in the source graph correspond to edge IDs in the target graph.
    edge_map: EM,
}

impl<'g, G>
    GraphCopier<
        'g,
        G,
        G::Directedness,
        G::EdgeMultiplicity,
        fn(&G::NodeData) -> (),
        fn(&G::EdgeData) -> (),
        (),
        (),
    >
where
    G: Graph + ?Sized,
{
    /// Creates a new `GraphCopier` for the given source graph with default transformations and no ID mappings.
    pub fn new(source: &'g G) -> Self {
        Self {
            source,
            directedness: source.directedness(),
            edge_multiplicity: source.edge_multiplicity(),
            node_transformer: |_| (),
            edge_transformer: |_| (),
            node_map: (),
            edge_map: (),
        }
    }
}

impl<'g, G, D, M, NT, ET, NM, EM> GraphCopier<'g, G, D, M, NT, ET, NM, EM>
where
    G: Graph + ?Sized,
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    /// Sets the directedness for the target graph, returning a new
    /// `GraphCopier`.  When using [`Self::copy_into`], this setting is ignored and
    /// the target graph's directedness will be used instead.
    pub fn with_directedness<D2>(
        self,
        directedness: D2,
    ) -> GraphCopier<'g, G, D2, M, NT, ET, NM, EM>
    where
        D2: DirectednessTrait,
    {
        GraphCopier {
            source: self.source,
            directedness,
            edge_multiplicity: self.edge_multiplicity,
            node_transformer: self.node_transformer,
            edge_transformer: self.edge_transformer,
            node_map: self.node_map,
            edge_map: self.edge_map,
        }
    }

    /// Sets the edge multiplicity for the target graph, returning a new
    /// `GraphCopier`.  When using [`Self::copy_into`], this setting is ignored and
    /// the target graph's edge multiplicity will be used instead.
    pub fn with_edge_multiplicity<M2>(
        self,
        multiplicity: M2,
    ) -> GraphCopier<'g, G, D, M2, NT, ET, NM, EM>
    where
        M2: EdgeMultiplicityTrait,
    {
        GraphCopier {
            source: self.source,
            directedness: self.directedness,
            edge_multiplicity: multiplicity,
            node_transformer: self.node_transformer,
            edge_transformer: self.edge_transformer,
            node_map: self.node_map,
            edge_map: self.edge_map,
        }
    }

    /// Sets a mapping for node IDs from the source graph to the target graph,
    /// returning a new `GraphCopier`.  This map will be populated during the
    /// copying process, allowing the caller to map node IDs in the source graph
    /// to the target graph.
    pub fn with_node_map<V>(
        self,
        node_map: &'g mut HashMap<G::NodeId, V>,
    ) -> GraphCopier<'g, G, D, M, NT, ET, &'g mut HashMap<G::NodeId, V>, EM> {
        GraphCopier {
            source: self.source,
            directedness: self.directedness,
            edge_multiplicity: self.edge_multiplicity,
            node_transformer: self.node_transformer,
            edge_transformer: self.edge_transformer,
            node_map,
            edge_map: self.edge_map,
        }
    }

    /// Sets a mapping for edge IDs from the source graph to the target graph,
    /// returning a new `GraphCopier`.  This map will be populated during the
    /// copying process, allowing the caller to map edge IDs in the source graph
    /// to the target graph.
    pub fn with_edge_map<V>(
        self,
        edge_map: &'g mut HashMap<G::EdgeId, V>,
    ) -> GraphCopier<'g, G, D, M, NT, ET, NM, &'g mut HashMap<G::EdgeId, V>> {
        GraphCopier {
            source: self.source,
            directedness: self.directedness,
            edge_multiplicity: self.edge_multiplicity,
            node_transformer: self.node_transformer,
            edge_transformer: self.edge_transformer,
            node_map: self.node_map,
            edge_map,
        }
    }

    /// Returns a new `GraphCopier` with a node transformer that clones the data.
    pub fn clone_nodes(
        self,
    ) -> GraphCopier<'g, G, D, M, fn(&G::NodeData) -> G::NodeData, ET, NM, EM>
    where
        G::NodeData: Clone,
    {
        self.transform_nodes(|n| n.clone())
    }

    /// Returns a new `GraphCopier` with an edge transformer that clones the data.
    pub fn clone_edges(
        self,
    ) -> GraphCopier<'g, G, D, M, NT, fn(&G::EdgeData) -> G::EdgeData, NM, EM>
    where
        G::EdgeData: Clone,
    {
        self.transform_edges(|e| e.clone())
    }

    /// Returns a new `GraphCopier` with the given node transformer function,
    /// which will be applied to each node's data during copying.
    pub fn transform_nodes<F, TN>(self, transformer: F) -> GraphCopier<'g, G, D, M, F, ET, NM, EM>
    where
        F: FnMut(&G::NodeData) -> TN,
    {
        GraphCopier {
            source: self.source,
            directedness: self.directedness,
            edge_multiplicity: self.edge_multiplicity,
            node_transformer: transformer,
            edge_transformer: self.edge_transformer,
            node_map: self.node_map,
            edge_map: self.edge_map,
        }
    }

    /// Returns a new `GraphCopier` with the given edge transformer function,
    /// which will be applied to each edge's data during copying.
    pub fn transform_edges<F, TE>(self, transformer: F) -> GraphCopier<'g, G, D, M, NT, F, NM, EM>
    where
        F: FnMut(&G::EdgeData) -> TE,
    {
        GraphCopier {
            source: self.source,
            directedness: self.directedness,
            edge_multiplicity: self.edge_multiplicity,
            node_transformer: self.node_transformer,
            edge_transformer: transformer,
            node_map: self.node_map,
            edge_map: self.edge_map,
        }
    }

    /// Consumes the `GraphCopier` and produces a new graph of type `T` with the
    /// copied data and structure and the specified transformations,
    /// directedness and edge multiplicity.  If you want to copy into an
    /// existing graph instead of creating a new one, use [`Self::copy_into`]
    /// instead.
    pub fn copy<T>(self) -> T
    where
        T: GraphMut,
        T::NodeId: 'g,
        T::EdgeId: 'g,
        T::Directedness: From<D>,
        T::EdgeMultiplicity: From<M>,
        NT: FnMut(&G::NodeData) -> T::NodeData,
        ET: FnMut(&G::EdgeData) -> T::EdgeData,
        NM: IntoHashMapRef<'g, G::NodeId, T::NodeId>,
        EM: IntoHashMapRef<'g, G::EdgeId, T::EdgeId>,
    {
        let mut target = T::new(self.directedness.into(), self.edge_multiplicity.into());
        self.copy_into(&mut target);
        target
    }

    /// Copies the data and structure from the source graph into the given target graph using
    /// the specified transformations, and the target graph's directedness and edge multiplicity.
    pub fn copy_into<T>(mut self, target: &mut T)
    where
        T: GraphMut,
        T::NodeId: 'g,
        T::EdgeId: 'g,
        NT: FnMut(&G::NodeData) -> T::NodeData,
        ET: FnMut(&G::EdgeData) -> T::EdgeData,
        NM: IntoHashMapRef<'g, G::NodeId, T::NodeId>,
        EM: IntoHashMapRef<'g, G::EdgeId, T::EdgeId>,
    {
        let node_map = match self.node_map.into_hash_map_ref() {
            Some(map) => map,
            None => &mut HashMap::new(),
        };

        // Copy all the nodes, saving them into a map.
        for node_id in self.source.node_ids() {
            let node_data = (self.node_transformer)(self.source.node_data(&node_id));
            let new_node_id = target.add_node(node_data);
            node_map.insert(node_id.clone(), new_node_id);
        }

        // Prepare edge maps if needed.
        let mut edge_maps = self
            .edge_map
            .into_hash_map_ref()
            .map(|m| (m, HashMap::new()));

        // Copy all the edges, using the node map to find the new source and target IDs.
        for edge_id in self.source.edge_ids() {
            let edge_data = (self.edge_transformer)(self.source.edge_data(&edge_id));
            let source_node_id = &node_map[&edge_id.left()];
            let target_node_id = &node_map[&edge_id.right()];
            let add_edge_result = target.add_edge(source_node_id, target_node_id, edge_data);

            // Maintain the edge map if the user provided one.
            if let Some((ref mut edge_map, ref mut reverse_edge_map)) = edge_maps {
                match add_edge_result {
                    AddEdgeResult::Added(new_edge_id) => {
                        edge_map.insert(edge_id.clone(), new_edge_id.clone());
                        reverse_edge_map.insert(new_edge_id, edge_id);
                    }
                    AddEdgeResult::Updated(old_edge_id, _) => {
                        edge_map.remove(&reverse_edge_map[&old_edge_id]);
                        edge_map.insert(edge_id.clone(), old_edge_id.clone());
                        reverse_edge_map.insert(old_edge_id, edge_id);
                    }
                }
            }
        }
    }
}

#[doc(hidden)]
pub trait IntoHashMapRef<'a, K, V> {
    fn into_hash_map_ref(self) -> Option<&'a mut HashMap<K, V>>;
}

impl<'a, K, V> IntoHashMapRef<'a, K, V> for () {
    fn into_hash_map_ref(self) -> Option<&'a mut HashMap<K, V>> {
        None
    }
}

impl<'a, K, V> IntoHashMapRef<'a, K, V> for &'a mut HashMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    fn into_hash_map_ref(self) -> Option<&'a mut HashMap<K, V>> {
        Some(self)
    }
}
