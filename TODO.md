[ ] Add more graph algorithms
[ ] Make graph algorithms returns paths
[x] Add graph ID to edge/node IDs
[ ] Add shrink_to_fit for adjacency matrices
[x] Add methods to EdgeId to find the NodeIds of its ends
[ ] Make sure top-level exports are as expected
[x] Test round-tripping of dot data parsing and generation for flat graphs
[ ] expose subgraph data to the dot parsing API
[ ] handle graph attributes and default node and edge attributes in DOT files
[ ] handle identifiers that correspond to DOT keywords
[ ] read "large graph" test data from a dot file instead and make sure generated graphs conform to it
[ ] Make a graph construction proxy trait for use with newtype wrappers around graph types, with default implementations of GraphMut methods that delegate to the inner graph
[x] add handling for data types defined at https://graphviz.org/doc/info/attrs.html
[ ] Propose adding a consuming iter_ones (e.g., into_iter_ones) to bitvec, and use it instead of a custom iterator
[x] Get rid of log2_size
[x] Support different edge multiplicities in LinkedGraph
[ ] Support different edge multiplicities in AdjacencyGraph
[x] Speed up/fix edge removal in AdjacencyGraph by clearing the row+column of a removed node in a single call
[x] Add a reflected adjacency bitmap for directed adjacency graphs
[x] Store regular and reflected data in a single bitvec
[ ] Clean up TODOs in code