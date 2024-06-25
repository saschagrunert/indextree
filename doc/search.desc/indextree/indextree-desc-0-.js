searchState.loadedDescShard("indextree", 0, "Arena based tree data structure\nAn iterator of the IDs of the ancestors of a given node.\nAttempt to append an ancestor node to a descendant.\nAttempt to append a node to itself.\nAn <code>Arena</code> structure containing certain <code>Node</code>s.\nAn iterator of the IDs of the children of a given node, in …\nTree printer for debugging.\nAn iterator of the IDs of a given node and its …\nIndicates that end of a node that has children.\nAn iterator of the IDs of the siblings after a given node.\nAttempt to insert a node after itself.\nAttempt to insert a node before itself.\nA node within a particular <code>Arena</code>.\nIndicator if the node is at a start or endpoint of the tree\nPossible node failures.\nA node identifier within a particular <code>Arena</code>.\nAn iterator of the IDs of the siblings before a given node.\nAn iterator of the IDs of the predecessors of a given node.\nAttempt to prepend an ancestor node to a descendant.\nAttempt to prepend a node to itself.\nAttempt to insert a removed node, or insert to a removed …\nAn iterator of the IDs of the children of a given node, in …\nAn iterator of the “sides” of a node visited during a …\nIndicates that start of a node that has children.\nAn iterator of the “sides” of a node visited during a …\nReturns an iterator of IDs of this node and its ancestors.\nAppends a new child to this node, after existing children.\nCreates and appends a new node (from its associated data) …\nReturns the number of nodes the arena can hold without …\nAppends a new child to this node, after existing children.\nInserts a new sibling after this node.\nInserts a new sibling before this node.\nPrepends a new child to this node, before existing …\nReturns an iterator of IDs of this node’s children.\nClears all the nodes in the arena, but retains its …\nCounts the number of nodes in arena and returns it.\nReturns the pretty-printable proxy object to the node and …\nAn iterator of the IDs of a given node and its …\nDetaches a node from its parent and siblings. Children are …\nReturns the ID of the first child of this node, unless it …\nReturns an iterator of IDs of this node and the siblings …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns a reference to the node with the given id if in …\nReturns a reference to the node data.\nReturns a mutable reference to the node with the given id …\nReturns a mutable reference to the node data.\nRetrieves the <code>NodeId</code> corresponding to a <code>Node</code> in the <code>Arena</code>.\nRetrieves the <code>NodeId</code> corresponding to the <code>Node</code> at <code>index</code> in …\nInserts a new sibling after this node.\nInserts a new sibling before this node.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns <code>true</code> if arena has no nodes, <code>false</code> otherwise.\nReturn if the <code>Node</code> of NodeId point to is removed.\nChecks if the node is marked as removed.\nReturns an iterator of all nodes in the arena in …\nReturns a mutable iterator of all nodes in the arena in …\nReturns the ID of the last child of this node, unless it …\nCreates a new empty <code>Arena</code>.\nCreates a new node from its associated data.\nReturns the ID of the next sibling of this node, unless it …\nReturns the next <code>NodeEdge</code> to be returned by forward …\nReturns the ID of the parent node, unless this node is the …\nReturns an iterator of IDs of this node and the siblings …\nReturns an iterator of IDs of this node and its …\nPrepends a new child to this node, before existing …\nReturns the previous <code>NodeEdge</code> to be returned by forward …\nReturns the ID of the previous sibling of this node, …\nRemoves a node from the arena.\nRemoves a node and its descendants from the arena.\nReserves capacity for <code>additional</code> more nodes to be inserted.\nReturns an iterator of IDs of this node’s children, in …\nAn iterator of the “sides” of a node visited during a …\nAn iterator of the “sides” of a node visited during a …\nCreates a new empty <code>Arena</code> with enough capacity to store <code>n</code> …")