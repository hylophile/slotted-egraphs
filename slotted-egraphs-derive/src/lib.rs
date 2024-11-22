use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::*;

// We allow the user to use tuples, Slot, Bind<_>, AppliedId and "user-defined types" in their enum variants.
// user-defined types will be understood as slot-independent constants, and ignored by the system.

#[proc_macro]
pub fn define_language(input: TokenStream1) -> TokenStream1 {
    let mut ie: ItemEnum = parse(input).unwrap();

    let name = ie.ident.clone();
    let str_names: Vec<Option<Expr>> = ie.variants.iter_mut()
                                      .map(|x| {
                                          x.discriminant.take().map(|(_, e)| e)
                                      }).collect();

    let ident_names: Vec<Ident> = ie.variants.iter().map(|x| x.ident.clone()).collect();
    let all_slot_occurrences_mut_arms: Vec<TokenStream2> = ie.variants.iter().map(|x| produce_all_slot_occurrences_mut(&name, x)).collect();
    let public_slot_occurrences_mut_arms: Vec<TokenStream2> = ie.variants.iter().map(|x| produce_public_slot_occurrences_mut(&name, x)).collect();
    let applied_id_occurrences_mut_arms: Vec<TokenStream2> = ie.variants.iter().map(|x| produce_applied_id_occurrences_mut(&name, x)).collect();
    let to_syntax_arms: Vec<TokenStream2> = ie.variants.iter().zip(&str_names).map(|(x, n)| produce_to_syntax(&name, &n, x)).collect();
    let from_syntax_arms1: Vec<TokenStream2> = ie.variants.iter().zip(&str_names).filter_map(|(x, n)| produce_from_syntax1(&name, &n, x)).collect();
    let from_syntax_arms2: Vec<TokenStream2> = ie.variants.iter().zip(&str_names).filter_map(|(x, n)| produce_from_syntax2(&name, &n, x)).collect();

    quote! {
        #[derive(PartialEq, Eq, Hash, Clone, Debug)]
        #ie

        impl Language for #name {
            fn all_slot_occurrences_mut(&mut self) -> Vec<&mut Slot> {
                match self {
                    #(#all_slot_occurrences_mut_arms),*
                }
            }
            fn public_slot_occurrences_mut(&mut self) -> Vec<&mut Slot> {
                match self {
                    #(#public_slot_occurrences_mut_arms),*
                }
            }
            fn applied_id_occurrences_mut(&mut self) -> Vec<&mut AppliedId> {
                match self {
                    #(#applied_id_occurrences_mut_arms),*
                }
            }

            fn to_syntax(&self) -> Vec<SyntaxElem> {
                match self {
                    #(#to_syntax_arms),*
                }
            }

            fn from_syntax(elems: &[SyntaxElem]) -> Option<Self> {
                let SyntaxElem::String(op) = elems.get(0)? else { return None };
                match &**op {
                    #(#from_syntax_arms1),*
                    _ => {
                        #(#from_syntax_arms2)*

                        None
                    },
                }
            }
        }
    }.to_token_stream().into()
}

fn produce_all_slot_occurrences_mut(name: &Ident, v: &Variant) -> TokenStream2 {
    let variant_name = &v.ident;
    let n = v.fields.len();
    let fields: Vec<Ident> = (0..n).map(|x| Ident::new(&format!("a{x}"), proc_macro2::Span::call_site())).collect();
    quote! {
        #name::#variant_name(#(#fields),*) => {
            let mut out: Vec<&mut Slot> = Vec::new();
            #(
                out.extend(#fields .all_slot_occurrences_iter_mut());
            )*
            out
        }
    }
}

fn produce_public_slot_occurrences_mut(name: &Ident, v: &Variant) -> TokenStream2 {
    let variant_name = &v.ident;
    let n = v.fields.len();
    let fields: Vec<Ident> = (0..n).map(|x| Ident::new(&format!("a{x}"), proc_macro2::Span::call_site())).collect();
    quote! {
        #name::#variant_name(#(#fields),*) => {
            let mut out: Vec<&mut Slot> = Vec::new();
            #(
                out.extend(#fields .public_slot_occurrences_iter_mut());
            )*
            out
        }
    }
}

fn produce_applied_id_occurrences_mut(name: &Ident, v: &Variant) -> TokenStream2 {
    let variant_name = &v.ident;
    let n = v.fields.len();
    let fields: Vec<Ident> = (0..n).map(|x| Ident::new(&format!("a{x}"), proc_macro2::Span::call_site())).collect();
    quote! {
        #name::#variant_name(#(#fields),*) => {
            let mut out: Vec<&mut AppliedId> = Vec::new();
            #(
                out.extend(#fields .applied_id_occurrences_iter_mut());
            )*
            out
        }
    }
}

fn produce_to_syntax(name: &Ident, e: &Option<Expr>, v: &Variant) -> TokenStream2 {
    let variant_name = &v.ident;

    if e.is_none() {
        return quote! {
            #name::#variant_name(a0) => {
                a0.to_syntax()
            }
        };
    }

    let e = e.as_ref().unwrap();
    let n = v.fields.len();
    let fields: Vec<Ident> = (0..n).map(|x| Ident::new(&format!("a{x}"), proc_macro2::Span::call_site())).collect();
    quote! {
        #name::#variant_name(#(#fields),*) => {
            let mut out: Vec<SyntaxElem> = vec![SyntaxElem::String(String::from(#e))];
            #(
                out.extend(#fields.to_syntax());
            )*
            out
        }
    }
}

fn produce_from_syntax1(name: &Ident, e: &Option<Expr>, v: &Variant) -> Option<TokenStream2> {
    let variant_name = &v.ident;

    let e = e.as_ref()?;
    let n = v.fields.len();
    let fields: Vec<Ident> = (0..n).map(|x| Ident::new(&format!("a{x}"), proc_macro2::Span::call_site())).collect();

    let types: Vec<Type> = v.fields.iter().map(|x| x.ty.clone()).collect();

    Some(quote! {
        #e => {
            let mut children = &elems[1..];
            let mut rest = children;
            #(
                let #fields = (0..=children.len()).filter_map(|n| {
                    let a = &children[..n];
                    rest = &children[n..];

                    <#types>::from_syntax(a)
                }).next()?;
                children = rest;
            )*
            Some(#name::#variant_name(#(#fields),*))
        }
    })
}

fn produce_from_syntax2(name: &Ident, e: &Option<Expr>, v: &Variant) -> Option<TokenStream2> {
    if e.is_some() { return None; }
    let variant_name = &v.ident;

    let ty = v.fields.iter().map(|x| x.ty.clone()).next().unwrap();
    Some(quote! {
        if let Some(a) = <#ty>::from_syntax(elems) {
            return Some(#name::#variant_name(a));
        }
    })
}
