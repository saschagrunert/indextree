use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Expr, Token, braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

#[derive(Clone)]
struct IndexNode {
    node: Expr,
    children: Punctuated<Self, Token![,]>,
}

impl Parse for IndexNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let node = input.parse::<Expr>()?;

        if input.parse::<Token![=>]>().is_err() {
            return Ok(IndexNode {
                node,
                children: Punctuated::new(),
            });
        }

        let children_stream;
        braced!(children_stream in input);
        let children = children_stream.parse_terminated(Self::parse, Token![,])?;

        Ok(IndexNode { node, children })
    }
}

#[derive(Clone)]
struct IndexTree {
    arena: Expr,
    root_node: Expr,
    nodes: Punctuated<IndexNode, Token![,]>,
}

impl Parse for IndexTree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let arena = input.parse::<Expr>()?;

        input.parse::<Token![,]>()?;

        let root_node = input.parse::<Expr>()?;

        let nodes = if input.parse::<Token![=>]>().is_ok() {
            let braced_nodes;
            braced!(braced_nodes in input);
            braced_nodes.parse_terminated(IndexNode::parse, Token![,])?
        } else {
            Punctuated::new()
        };

        let _ = input.parse::<Token![,]>();

        Ok(IndexTree {
            arena,
            root_node,
            nodes,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionKind {
    Append,
    Parent,
    Nest,
}

enum Action {
    Append(Expr),
    Parent,
    Nest,
}

impl Action {
    fn kind(&self) -> ActionKind {
        match self {
            Action::Append(_) => ActionKind::Append,
            Action::Parent => ActionKind::Parent,
            Action::Nest => ActionKind::Nest,
        }
    }

    fn to_stream(&self) -> TokenStream {
        match self {
            Action::Append(expr) => quote! {
                __last = __node.append_value(#expr, __arena);
            },
            Action::Parent => quote! {
                let __temp = ::indextree::Arena::get(__arena, __node);
                let __temp = ::core::option::Option::unwrap(__temp);
                let __temp = ::indextree::Node::parent(__temp);
                let __temp = ::core::option::Option::unwrap(__temp);
                __node = __temp;
            },
            Action::Nest => quote! {
                __node = __last;
            },
        }
    }
}

enum StackItem {
    Node(Box<IndexNode>),
    NestingMarker,
}

struct ActionStream {
    count: usize,
    kind: ActionKind,
    stream: TokenStream,
}

impl ToTokens for ActionStream {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.stream.clone());
    }
}

/// Construct a tree for a given arena.
///
/// This macro creates a tree in an [`Arena`] with a pre-defined layout. If the root node is of
/// type [`NodeId`], then that [`NodeId`] is used for the root node, but if it's any other type,
/// then it creates a new root node on-the-fly. The macro returns [`NodeId`] of the root node.
///
/// # Examples
///
/// ```
/// # use indextree::{Arena, macros::tree};
/// # let mut arena = Arena::new();
/// let root_node = arena.new_node("root node");
/// tree!(
///     &mut arena,
///     root_node => {
///         "1",
///         "2" => {
///             "2_1" => { "2_1_1" },
///             "2_2",
///         },
///         "3",
///     }
/// );
///
/// let automagical_root_node = tree!(
///     &mut arena,
///     "root node, but automagically created" => {
///         "1",
///         "2" => {
///             "2_1" => { "2_1_1" },
///             "2_2",
///         },
///         "3",
///     }
/// );
/// ```
///
/// Note that you can anchor the root node in the macro to any node at any nesting. So you can take
/// an already existing node of a tree and attach another tree to it:
/// ```
/// # use indextree::{Arena, macros::tree};
/// # let mut arena = Arena::new();
/// let root_node = tree!(
///     &mut arena,
///     "root node" => {
///         "1",
///         "2",
///         "3",
///     }
/// );
///
/// let node_1 = arena.get(root_node).unwrap().first_child().unwrap();
/// let node_2 = arena.get(node_1).unwrap().next_sibling().unwrap();
/// tree!(
///     &mut arena,
///     node_2 => {
///         "2_1" => { "2_1_1" },
///         "2_2",
///     }
/// );
/// ```
///
/// It is also possible to create an empty root_node, although, I'm not sure why you'd want to do
/// that.
/// ```
/// # use indextree::{Arena, macros::tree};
/// # let mut arena = Arena::new();
/// let root_node = tree!(
///     &mut arena,
///     "my root node",
/// );
/// ```
/// Empty nodes can also be defined as `=> {}`
/// ```
/// # use indextree::{Arena, macros::tree};
/// # let mut arena = Arena::new();
/// let root_node = tree!(
///     &mut arena,
///     "my root node" => {},
/// );
/// ```
///
/// [`Arena`]: https://docs.rs/indextree/latest/indextree/struct.Arena.html
/// [`NodeId`]: https://docs.rs/indextree/latest/indextree/struct.NodeId.html
#[proc_macro]
pub fn tree(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let IndexTree {
        arena,
        root_node,
        nodes,
    } = parse_macro_input!(input as IndexTree);

    let mut stack: Vec<StackItem> = nodes
        .into_iter()
        .map(|n| StackItem::Node(Box::new(n)))
        .rev()
        .collect();

    let mut action_buffer: Vec<Action> = Vec::new();

    while let Some(item) = stack.pop() {
        let StackItem::Node(index_node) = item else {
            action_buffer.push(Action::Parent);
            continue;
        };

        action_buffer.push(Action::Append(index_node.node));

        if index_node.children.is_empty() {
            continue;
        }

        stack.push(StackItem::NestingMarker);
        action_buffer.push(Action::Nest);
        stack.extend(
            index_node
                .children
                .into_iter()
                .map(|n| StackItem::Node(Box::new(n)))
                .rev(),
        );
    }

    // Coalesce consecutive actions of the same kind.
    let mut actions: Vec<ActionStream> = Vec::new();
    for action in &action_buffer {
        let kind = action.kind();
        let stream = action.to_stream();
        if matches!(actions.last(), Some(last) if last.kind == kind) {
            let last = actions.last_mut().unwrap();
            last.count += 1;
            last.stream.extend(stream);
            continue;
        }
        actions.push(ActionStream {
            count: 1,
            kind,
            stream,
        });
    }

    // Remove trailing Parent actions (they're unnecessary).
    if actions
        .last()
        .is_some_and(|last| last.kind == ActionKind::Parent)
    {
        actions.pop();
    }

    // HACK(alexmozaidze): Due to the fact that specialization is unstable, we must resort to
    // autoref specialization trick.
    // https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md
    quote! {{
        let mut __arena: &mut ::indextree::Arena<_> = #arena;

        #[repr(transparent)]
        struct __Wrapping<__T>(::core::mem::ManuallyDrop<__T>);

        trait __ToNodeId<__T> {
            fn __to_node_id(&mut self, __arena: &mut ::indextree::Arena<__T>) -> ::indextree::NodeId;
        }

        trait __NodeIdToNodeId<__T> {
            fn __to_node_id(&mut self, __arena: &mut ::indextree::Arena<__T>) -> ::indextree::NodeId;
        }

        impl<__T> __NodeIdToNodeId<__T> for __Wrapping<::indextree::NodeId> {
            fn __to_node_id(&mut self, __arena: &mut ::indextree::Arena<__T>) -> ::indextree::NodeId {
                unsafe { ::core::mem::ManuallyDrop::take(&mut self.0) }
            }
        }

        impl<__T> __ToNodeId<__T> for &mut __Wrapping<__T> {
            fn __to_node_id(&mut self, __arena: &mut ::indextree::Arena<__T>) -> ::indextree::NodeId {
                ::indextree::Arena::new_node(__arena, unsafe { ::core::mem::ManuallyDrop::take(&mut self.0) })
            }
        }

        let __root_node: ::indextree::NodeId = {
            let mut __root_node = __Wrapping(::core::mem::ManuallyDrop::new(#root_node));
            (&mut __root_node).__to_node_id(__arena)
        };
        let mut __node: ::indextree::NodeId = __root_node;
        let mut __last: ::indextree::NodeId;

        #(#actions)*

        __root_node
    }}.into()
}
