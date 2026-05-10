[ ] Add more graph algorithms
[ ] Add shrink_to_fit for adjacency matrices
[ ] Clean up TODOs in code
[ ] expose subgraph data to the dot parsing API
[ ] handle graph attributes and default node and edge attributes in DOT files
[ ] handle identifiers that correspond to DOT keywords
[ ] Make a graph construction proxy trait for use with newtype wrappers around graph types, with default implementations of GraphMut methods that delegate to the inner graph
[ ] Make graph algorithms returns paths
[ ] Make sure top-level exports are as expected
[ ] Propose adding a consuming iter_ones (e.g., into_iter_ones) to bitvec, and use it instead of a custom iterator
[ ] read "large graph" test data from a dot file instead and make sure generated graphs conform to it
[ ] Support different edge multiplicities in AdjacencyGraph

[x] Add a reflected adjacency bitmap for directed adjacency graphs
[x] Add graph ID to edge/node IDs
[x] add handling for data types defined at https://graphviz.org/doc/info/attrs.html
[x] Add methods to EdgeId to find the NodeIds of its ends
[x] Get rid of log2_size
[x] Speed up/fix edge removal in AdjacencyGraph by clearing the row+column of a removed node in a single call
[x] Store regular and reflected data in a single bitvec
[x] Support different edge multiplicities in LinkedGraph
[x] Test round-tripping of dot data parsing and generation for flat graphs
