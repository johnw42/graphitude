[ ] Add more graph algorithms
[ ] Make graph algorithms returns paths
[x] Add graph ID to edge/node IDs
[ ] Add shrink_to_fit for adjacency matrices
[x] Add methods to EdgeId to find the NodeIds of its ends
[ ] Make sure top-level exports are as expected
[ ] Test round-tripping of dot data parsing and generation for flat graphs
[ ] expose subgraph data to the dot parsing API
[ ] read "large graph" test data from a dot file instead and make sure generated graphs conform to it
[ ] Get rid of the dot builder type and move the functionality to a trait implemented by the graph itself
[ ] Make a graph construction proxy trait for use with newtype wrappers around graph types, with default implementations of GraphMut methods that delegate to the inner graph
[x] add handling for data types defined at https://graphviz.org/doc/info/attrs.html
[ ] Propose adding a consuming iter_ones (e.g., into_iter_ones) to bitvec
