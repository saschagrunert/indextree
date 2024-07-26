use either::Either;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use strum::EnumDiscriminants;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Token,
};

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, EnumDiscriminants, Debug)]
#[strum_discriminants(name(ActionKind))]
enum Action {
    Append(Expr),
    Parent,
    Nest,
}

impl ToTokens for Action {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_stream())
    }
}

impl Action {
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

#[derive(Clone, Debug)]
struct NestingLevelMarker;

#[derive(Clone, Debug)]
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

    let mut stack: Vec<Either<_, NestingLevelMarker>> =
        nodes.into_iter().map(Either::Left).rev().collect();

    let mut action_buffer: Vec<Action> = Vec::new();

    while let Some(item) = stack.pop() {
        let Either::Left(IndexNode { node, children }) = item else {
            action_buffer.push(Action::Parent);
            continue;
        };

        action_buffer.push(Action::Append(node));

        if children.is_empty() {
            continue;
        }

        // going one level deeper
        stack.push(Either::Right(NestingLevelMarker));
        action_buffer.push(Action::Nest);
        stack.extend(children.into_iter().map(Either::Left).rev());
    }

    let mut actions: Vec<ActionStream> = action_buffer
        .into_iter()
        .map(|action| ActionStream {
            count: 1,
            kind: ActionKind::from(&action),
            stream: action.to_stream(),
        })
        .coalesce(|action1, action2| {
            if action1.kind != action2.kind {
                return Err((action1, action2));
            }

            let count = action1.count + action2.count;
            let kind = action1.kind;
            let mut stream = action1.stream;
            stream.extend(action2.stream);
            Ok(ActionStream {
                count,
                kind,
                stream,
            })
        })
        .collect();

    let is_last_action_useless = actions
        .last()
        .map(|last| last.kind == ActionKind::Parent)
        .unwrap_or(false);
    if is_last_action_useless {
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
