var N=null,E="",T="t",U="u",searchIndex={};
var R=["An iterator of references to a given node and its…","nodeid","option","ancestors","Returns an iterator of references to this node and the…","arena","children","Returns an iterator of references to this node and its…","descendants","traverse","Appends a new child to this node, after existing children.","nodeerror","Prepends a new child to this node, before existing children.","Inserts a new sibling after this node.","Inserts a new sibling before this node.","result","to_owned","clone_into","try_from","try_into","borrow_mut","to_string","type_id","into_iter","borrow","typeid","nodeedge","precedingsiblings","followingsiblings","reversechildren","reversetraverse","ordering","formatter","NodeError","NodeEdge","Ancestors","Children","Descendants","FollowingSiblings","PrecedingSiblings","ReverseChildren","ReverseTraverse","Traverse"];

searchIndex["indextree"]={"doc":"Arena based tree data structure","i":[[3,"Arena","indextree","An `Arena` structure containing certain [`Node`]s.",N,N],[3,"NodeId",E,"A node identifier within a particular [`Arena`].",N,N],[3,"Node",E,"A node within a particular `Arena`.",N,N],[12,"data",E,"The actual data which will be stored within the tree.",0,N],[3,R[35],E,"An iterator of references to the ancestors a given node.",N,N],[3,R[36],E,"An iterator of references to the children of a given node.",N,N],[3,R[37],E,R[0],N,N],[3,R[38],E,"An iterator of references to the siblings after a given…",N,N],[3,R[39],E,"An iterator of references to the siblings before a given…",N,N],[3,R[40],E,"An iterator of references to the children of a given node,…",N,N],[3,R[41],E,R[0],N,N],[3,R[42],E,R[0],N,N],[4,R[33],E,"Possible node failures.",N,N],[13,"AppendSelf",E,"Attempt to append a node to itself.",1,N],[13,"PrependSelf",E,"Attempt to prepend a node to itself.",1,N],[13,"InsertBeforeSelf",E,"Attempt to insert a node before itself.",1,N],[13,"InsertAfterSelf",E,"Attempt to insert a node after itself.",1,N],[13,"Removed",E,"Attempt to insert a removed node, or insert to a removed…",1,N],[4,R[34],E,"Indicator if the node is at a start or endpoint of the tree",N,N],[13,"Start",E,"Indicates that start of a node that has children.",2,N],[13,"End",E,"Indicates that end of a node that has children.",2,N],[11,"new",E,"Creates a new empty `Arena`.",3,[[],[R[5]]]],[11,"new_node",E,"Creates a new node from its associated data.",3,[[["self"],[T]],[R[1]]]],[11,"count",E,"Counts the number of nodes in arena and returns it.",3,[[["self"]],["usize"]]],[11,"is_empty",E,"Returns `true` if arena has no nodes, `false` otherwise.",3,[[["self"]],["bool"]]],[11,"get",E,"Returns a reference to the node with the given id if in…",3,[[["self"],[R[1]]],[[R[2],["node"]],["node"]]]],[11,"get_mut",E,"Returns a mutable reference to the node with the given id…",3,[[["self"],[R[1]]],[["node"],[R[2],["node"]]]]],[11,"iter",E,"Returns an iterator of all nodes in the arena in…",3,[[["self"]]]],[11,R[3],E,R[7],4,[[[R[5]]],[R[3]]]],[11,"preceding_siblings",E,R[4],4,[[[R[5]]],[R[27]]]],[11,"following_siblings",E,R[4],4,[[[R[5]]],[R[28]]]],[11,R[6],E,"Returns an iterator of references to this node’s children.",4,[[[R[5]]],[R[6]]]],[11,"reverse_children",E,"Returns an iterator of references to this node’s children,…",4,[[[R[5]]],[R[29]]]],[11,R[8],E,R[7],4,[[[R[5]]],[R[8]]]],[11,R[9],E,R[7],4,[[[R[5]]],[R[9]]]],[11,"reverse_traverse",E,R[7],4,[[[R[5]]],[R[30]]]],[11,"detach",E,"Detaches a node from its parent and siblings. Children are…",4,[[[R[5]]]]],[11,"append",E,R[10],4,[[[R[1]],[R[5]]]]],[11,"checked_append",E,R[10],4,[[[R[1]],[R[5]]],[[R[11]],[R[15],[R[11]]]]]],[11,"prepend",E,R[12],4,[[[R[1]],[R[5]]]]],[11,"checked_prepend",E,R[12],4,[[[R[1]],[R[5]]],[[R[11]],[R[15],[R[11]]]]]],[11,"insert_after",E,R[13],4,[[[R[1]],[R[5]]]]],[11,"checked_insert_after",E,R[13],4,[[[R[1]],[R[5]]],[[R[11]],[R[15],[R[11]]]]]],[11,"insert_before",E,R[14],4,[[[R[1]],[R[5]]]]],[11,"checked_insert_before",E,R[14],4,[[[R[1]],[R[5]]],[[R[11]],[R[15],[R[11]]]]]],[11,"remove",E,"Removes a node from the arena.",4,[[[R[5]]]]],[11,"parent",E,"Returns the ID of the parent node, unless this node is the…",0,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"first_child",E,"Returns the ID of the first child of this node, unless it…",0,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"last_child",E,"Returns the ID of the last child of this node, unless it…",0,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"previous_sibling",E,"Returns the ID of the previous sibling of this node,…",0,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"next_sibling",E,"Returns the ID of the next sibling of this node, unless it…",0,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"is_removed",E,"Checks if the node is marked as removed.",0,[[["self"]],["bool"]]],[11,"from",E,E,3,[[[T]],[T]]],[11,"into",E,E,3,[[],[U]]],[11,R[16],E,E,3,[[["self"]],[T]]],[11,R[17],E,E,3,[[[T],["self"]]]],[11,R[18],E,E,3,[[[U]],[R[15]]]],[11,R[19],E,E,3,[[],[R[15]]]],[11,R[20],E,E,3,[[["self"]],[T]]],[11,R[24],E,E,3,[[["self"]],[T]]],[11,R[22],E,E,3,[[["self"]],[R[25]]]],[11,R[21],E,E,4,[[["self"]],["string"]]],[11,"from",E,E,4,[[[T]],[T]]],[11,"into",E,E,4,[[],[U]]],[11,R[16],E,E,4,[[["self"]],[T]]],[11,R[17],E,E,4,[[[T],["self"]]]],[11,R[18],E,E,4,[[[U]],[R[15]]]],[11,R[19],E,E,4,[[],[R[15]]]],[11,R[20],E,E,4,[[["self"]],[T]]],[11,R[24],E,E,4,[[["self"]],[T]]],[11,R[22],E,E,4,[[["self"]],[R[25]]]],[11,R[21],E,E,0,[[["self"]],["string"]]],[11,"from",E,E,0,[[[T]],[T]]],[11,"into",E,E,0,[[],[U]]],[11,R[16],E,E,0,[[["self"]],[T]]],[11,R[17],E,E,0,[[[T],["self"]]]],[11,R[18],E,E,0,[[[U]],[R[15]]]],[11,R[19],E,E,0,[[],[R[15]]]],[11,R[20],E,E,0,[[["self"]],[T]]],[11,R[24],E,E,0,[[["self"]],[T]]],[11,R[22],E,E,0,[[["self"]],[R[25]]]],[11,"from",E,E,5,[[[T]],[T]]],[11,"into",E,E,5,[[],[U]]],[11,R[23],E,E,5,[[],["i"]]],[11,R[16],E,E,5,[[["self"]],[T]]],[11,R[17],E,E,5,[[[T],["self"]]]],[11,R[18],E,E,5,[[[U]],[R[15]]]],[11,R[19],E,E,5,[[],[R[15]]]],[11,R[20],E,E,5,[[["self"]],[T]]],[11,R[24],E,E,5,[[["self"]],[T]]],[11,R[22],E,E,5,[[["self"]],[R[25]]]],[11,"from",E,E,6,[[[T]],[T]]],[11,"into",E,E,6,[[],[U]]],[11,R[23],E,E,6,[[],["i"]]],[11,R[16],E,E,6,[[["self"]],[T]]],[11,R[17],E,E,6,[[[T],["self"]]]],[11,R[18],E,E,6,[[[U]],[R[15]]]],[11,R[19],E,E,6,[[],[R[15]]]],[11,R[20],E,E,6,[[["self"]],[T]]],[11,R[24],E,E,6,[[["self"]],[T]]],[11,R[22],E,E,6,[[["self"]],[R[25]]]],[11,"from",E,E,7,[[[T]],[T]]],[11,"into",E,E,7,[[],[U]]],[11,R[23],E,E,7,[[],["i"]]],[11,R[16],E,E,7,[[["self"]],[T]]],[11,R[17],E,E,7,[[[T],["self"]]]],[11,R[18],E,E,7,[[[U]],[R[15]]]],[11,R[19],E,E,7,[[],[R[15]]]],[11,R[20],E,E,7,[[["self"]],[T]]],[11,R[24],E,E,7,[[["self"]],[T]]],[11,R[22],E,E,7,[[["self"]],[R[25]]]],[11,"from",E,E,8,[[[T]],[T]]],[11,"into",E,E,8,[[],[U]]],[11,R[23],E,E,8,[[],["i"]]],[11,R[16],E,E,8,[[["self"]],[T]]],[11,R[17],E,E,8,[[[T],["self"]]]],[11,R[18],E,E,8,[[[U]],[R[15]]]],[11,R[19],E,E,8,[[],[R[15]]]],[11,R[20],E,E,8,[[["self"]],[T]]],[11,R[24],E,E,8,[[["self"]],[T]]],[11,R[22],E,E,8,[[["self"]],[R[25]]]],[11,"from",E,E,9,[[[T]],[T]]],[11,"into",E,E,9,[[],[U]]],[11,R[23],E,E,9,[[],["i"]]],[11,R[16],E,E,9,[[["self"]],[T]]],[11,R[17],E,E,9,[[[T],["self"]]]],[11,R[18],E,E,9,[[[U]],[R[15]]]],[11,R[19],E,E,9,[[],[R[15]]]],[11,R[20],E,E,9,[[["self"]],[T]]],[11,R[24],E,E,9,[[["self"]],[T]]],[11,R[22],E,E,9,[[["self"]],[R[25]]]],[11,"from",E,E,10,[[[T]],[T]]],[11,"into",E,E,10,[[],[U]]],[11,R[23],E,E,10,[[],["i"]]],[11,R[16],E,E,10,[[["self"]],[T]]],[11,R[17],E,E,10,[[[T],["self"]]]],[11,R[18],E,E,10,[[[U]],[R[15]]]],[11,R[19],E,E,10,[[],[R[15]]]],[11,R[20],E,E,10,[[["self"]],[T]]],[11,R[24],E,E,10,[[["self"]],[T]]],[11,R[22],E,E,10,[[["self"]],[R[25]]]],[11,"from",E,E,11,[[[T]],[T]]],[11,"into",E,E,11,[[],[U]]],[11,R[23],E,E,11,[[],["i"]]],[11,R[16],E,E,11,[[["self"]],[T]]],[11,R[17],E,E,11,[[[T],["self"]]]],[11,R[18],E,E,11,[[[U]],[R[15]]]],[11,R[19],E,E,11,[[],[R[15]]]],[11,R[20],E,E,11,[[["self"]],[T]]],[11,R[24],E,E,11,[[["self"]],[T]]],[11,R[22],E,E,11,[[["self"]],[R[25]]]],[11,"from",E,E,12,[[[T]],[T]]],[11,"into",E,E,12,[[],[U]]],[11,R[23],E,E,12,[[],["i"]]],[11,R[16],E,E,12,[[["self"]],[T]]],[11,R[17],E,E,12,[[[T],["self"]]]],[11,R[18],E,E,12,[[[U]],[R[15]]]],[11,R[19],E,E,12,[[],[R[15]]]],[11,R[20],E,E,12,[[["self"]],[T]]],[11,R[24],E,E,12,[[["self"]],[T]]],[11,R[22],E,E,12,[[["self"]],[R[25]]]],[11,R[21],E,E,1,[[["self"]],["string"]]],[11,"from",E,E,1,[[[T]],[T]]],[11,"into",E,E,1,[[],[U]]],[11,R[16],E,E,1,[[["self"]],[T]]],[11,R[17],E,E,1,[[[T],["self"]]]],[11,R[18],E,E,1,[[[U]],[R[15]]]],[11,R[19],E,E,1,[[],[R[15]]]],[11,R[20],E,E,1,[[["self"]],[T]]],[11,R[24],E,E,1,[[["self"]],[T]]],[11,R[22],E,E,1,[[["self"]],[R[25]]]],[11,"from",E,E,2,[[[T]],[T]]],[11,"into",E,E,2,[[],[U]]],[11,R[16],E,E,2,[[["self"]],[T]]],[11,R[17],E,E,2,[[[T],["self"]]]],[11,R[18],E,E,2,[[[U]],[R[15]]]],[11,R[19],E,E,2,[[],[R[15]]]],[11,R[20],E,E,2,[[["self"]],[T]]],[11,R[24],E,E,2,[[["self"]],[T]]],[11,R[22],E,E,2,[[["self"]],[R[25]]]],[11,"default",E,E,3,[[],["self"]]],[11,"next",E,E,5,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"next",E,E,9,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"next",E,E,8,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"next",E,E,6,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"next",E,E,10,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"next",E,E,7,[[["self"]],[[R[2],[R[1]]],[R[1]]]]],[11,"next",E,E,12,[[["self"]],[[R[2],[R[26]]],[R[26]]]]],[11,"next",E,E,11,[[["self"]],[[R[2],[R[26]]],[R[26]]]]],[11,"clone",E,E,3,[[["self"]],[R[5]]]],[11,"clone",E,E,1,[[["self"]],[R[11]]]],[11,"clone",E,E,4,[[["self"]],[R[1]]]],[11,"clone",E,E,0,[[["self"]],["node"]]],[11,"clone",E,E,5,[[["self"]],[R[3]]]],[11,"clone",E,E,9,[[["self"]],[R[27]]]],[11,"clone",E,E,8,[[["self"]],[R[28]]]],[11,"clone",E,E,6,[[["self"]],[R[6]]]],[11,"clone",E,E,10,[[["self"]],[R[29]]]],[11,"clone",E,E,7,[[["self"]],[R[8]]]],[11,"clone",E,E,2,[[["self"]],[R[26]]]],[11,"clone",E,E,12,[[["self"]],[R[9]]]],[11,"clone",E,E,11,[[["self"]],[R[30]]]],[11,"cmp",E,E,4,[[["self"],[R[1]]],[R[31]]]],[11,"partial_cmp",E,E,4,[[["self"],[R[1]]],[[R[2],[R[31]]],[R[31]]]]],[11,"lt",E,E,4,[[["self"],[R[1]]],["bool"]]],[11,"le",E,E,4,[[["self"],[R[1]]],["bool"]]],[11,"gt",E,E,4,[[["self"],[R[1]]],["bool"]]],[11,"ge",E,E,4,[[["self"],[R[1]]],["bool"]]],[11,"eq",E,E,3,[[["self"],[R[5]]],["bool"]]],[11,"ne",E,E,3,[[["self"],[R[5]]],["bool"]]],[11,"eq",E,E,4,[[["self"],[R[1]]],["bool"]]],[11,"ne",E,E,4,[[["self"],[R[1]]],["bool"]]],[11,"eq",E,E,0,[[["self"],["node"]],["bool"]]],[11,"ne",E,E,0,[[["self"],["node"]],["bool"]]],[11,"eq",E,E,2,[[["self"],[R[26]]],["bool"]]],[11,"ne",E,E,2,[[["self"],[R[26]]],["bool"]]],[11,"fmt",E,E,3,[[["self"],[R[32]]],[R[15]]]],[11,"fmt",E,E,1,[[["self"],[R[32]]],[R[15]]]],[11,"fmt",E,E,4,[[["self"],[R[32]]],[R[15]]]],[11,"fmt",E,E,0,[[["self"],[R[32]]],[R[15]]]],[11,"fmt",E,E,2,[[["self"],[R[32]]],[R[15]]]],[11,"fmt",E,E,1,[[["self"],[R[32]]],[R[15]]]],[11,"fmt",E,E,4,[[["self"],[R[32]]],[R[15]]]],[11,"fmt",E,E,0,[[["self"],[R[32]]],[R[15]]]],[11,"index",E,E,3,[[["self"],[R[1]]],["node"]]],[11,"index_mut",E,E,3,[[["self"],[R[1]]],["node"]]],[11,"hash",E,E,4,[[["self"],["__h"]]]],[11,"hash",E,E,2,[[["self"],["__h"]]]]],"p":[[3,"Node"],[4,R[33]],[4,R[34]],[3,"Arena"],[3,"NodeId"],[3,R[35]],[3,R[36]],[3,R[37]],[3,R[38]],[3,R[39]],[3,R[40]],[3,R[41]],[3,R[42]]]};
initSearch(searchIndex);addSearchOptions(searchIndex);