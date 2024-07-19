use either::Either;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, punctuated::Punctuated, Token};

#[derive(Clone, Debug)]
struct IndexNode {
    node: syn::Expr,
    children: Punctuated<Self, Token![,]>,
}

impl Parse for IndexNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let node = input.parse::<syn::Expr>()?;

        if input.parse::<Token![=>]>().is_err() {
            return Ok(IndexNode {
                node,
                children: Punctuated::new(),
            });
        }

        let children_stream;
        syn::braced!(children_stream in input);
        let children = children_stream.parse_terminated(Self::parse, Token![,])?;

        Ok(IndexNode { node, children })
    }
}

#[derive(Clone, Debug)]
struct IndexTree {
    arena: syn::Expr,
    root_node: syn::Expr,
    nodes: Punctuated<IndexNode, Token![,]>,
}

impl Parse for IndexTree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let arena = input.parse::<syn::Expr>()?;

        input.parse::<Token![,]>()?;

        let root_node = input.parse::<syn::Expr>()?;

        input.parse::<Token![=>]>()?;

        let nodes;
        syn::braced!(nodes in input);
        let nodes = nodes.parse_terminated(IndexNode::parse, Token![,])?;

        Ok(IndexTree { arena, root_node, nodes })
    }
}

#[derive(Clone, Debug)]
struct NestingLevelMarker;

#[proc_macro]
pub fn tree(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let IndexTree { arena, root_node, nodes } = parse_macro_input!(input as IndexTree);

    let mut stack: Vec<Either<_, NestingLevelMarker>> = nodes
        .into_iter()
        .map(Either::Left)
        .rev()
        .collect();

    // HACK: Due to the fact that specialization is unstable, we must resort to manual type
    // checking and transmuting the value once the type is checked, in order to satisfy the borrow
    // checker. Also, using `std::any::Any` trait is no use, since it requires dynamic dispatch,
    // which is not zero-cost.
    let mut action_buffer = quote! {
        let mut __arena: &mut ::indextree::Arena<_> = #arena;
        let __root_node: ::indextree::NodeId = {
            let __root_node = #root_node;
            if ::std::any::type_name_of_val(&__root_node) == ::std::any::type_name::<::indextree::NodeId>() {
                let __root_node = unsafe { ::std::mem::transmute::<_, ::indextree::NodeId>(__root_node) };
                __root_node
            } else {
                let __root_node = unsafe { ::std::mem::transmute::<_, _>(__root_node) };
                __arena.new_node(__root_node)
            }
        };
        let mut __node: ::indextree::NodeId = __root_node;
        let mut __last: ::indextree::NodeId;
    };

    while let Some(item) = stack.pop() {
        let Either::Left(IndexNode { node, children }) = item else {
            action_buffer.extend(quote! {
                __node = __arena.get(__node).unwrap().parent().unwrap();
            });
            continue;
        };

        action_buffer.extend(quote! {
            __last = __node.append_value(#node, __arena);
        });

        if children.is_empty() {
            continue;
        }

        // going one level deeper
        stack.push(Either::Right(NestingLevelMarker));
        action_buffer.extend(quote! {
            __node = __last;
        });
        stack.extend(children.into_iter().map(Either::Left).rev());
    }

    quote! {{
        #action_buffer;
        __root_node
    }}.into()
}
