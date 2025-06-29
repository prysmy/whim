use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{ItemFn, ItemStruct, parse_macro_input};

#[proc_macro_derive(Entity, attributes(id))]
pub fn derive_entity(item: TokenStream) -> TokenStream {
    let ItemStruct { ident, fields, .. } = parse_macro_input!(item as ItemStruct);

    let id_field = fields
        .iter()
        .enumerate()
        .find(|(_, f)| f.attrs.iter().any(|a| a.path().is_ident("id")))
        .map(|(pos, f)| f.ident.clone().unwrap_or_else(|| format_ident!("{}", pos)));

    let Some(id_field) = id_field else {
        return syn::Error::new_spanned(
            fields,
            "Entity must have a field with the `#[id]` attribute",
        )
        .to_compile_error()
        .into();
    };

    quote! {
        impl whim::prelude::Entity for #ident {
            fn get_id(&self) -> &whim::prelude::Id<Self> {
                &self.#id_field
            }
        }
    }
    .into()
}

#[proc_macro_derive(Searchable, attributes(search))]
pub fn derive_searchable(item: TokenStream) -> TokenStream {
    // TODO support for enums
    let ItemStruct { ident, fields, .. } = parse_macro_input!(item as ItemStruct);

    let supported_fields = fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("search")))
        .enumerate()
        .map(|(pos, f)| f.ident.clone().unwrap_or_else(|| format_ident!("{}", pos)))
        .collect::<Vec<_>>();

    let index_statements = supported_fields.iter().map(|field| {
        quote! {
            self.#field.index(indexer);
        }
    });

    let score_statements = supported_fields
        .iter()
        .map(|field| {
            quote! {
                self.#field.get_score(searcher)
            }
        })
        .collect::<Vec<_>>();

    let get_score = if score_statements.is_empty() {
        quote! {
            None
        }
    } else {
        quote! {
            let items = vec![#(#score_statements),*]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            if !items.is_empty() {
                Some(items.into_iter().fold(0f32, f32::max))
            } else {
                None
            }
        }
    };

    quote! {
        impl whim::search::Searchable for #ident {
            fn index(&self, indexer: &mut whim::search::NgramIndexer) {
                #(#index_statements)*
            }

            fn get_score(&self, searcher: &whim::search::BitapSearcher) -> Option<f32> {
                #get_score
            }
        }
    }
    .into()
}

struct IndexerAttr {
    index_name: syn::Ident,
    entity: syn::Ident,
}

impl syn::parse::Parse for IndexerAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let index_name: syn::Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?; // Expect a comma after the
        let entity: syn::Ident = input.parse()?;
        Ok(IndexerAttr { index_name, entity })
    }
}

#[proc_macro_attribute]
pub fn index(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let IndexerAttr { index_name, entity } = syn::parse_macro_input!(attrs as IndexerAttr);

    let vis = &function.vis;
    let function_name = &function.sig.ident;

    // Kinda hacky way to check if the function returns a Vec<Index> or not
    let is_vec = function
        .sig
        .output
        .to_token_stream()
        .to_string()
        .contains("Vec");

    let return_statement = if is_vec {
        quote! {
            #function_name(entity).iter().map(|index| {
                whim::indices::Index::from(index)
            }).collect()
        }
    } else {
        quote! {
            vec![whim::indices::Index::from(#function_name(entity))]
        }
    };

    quote! {
        #vis struct #index_name;

        impl whim::indices::Indexer for #index_name {
            type Entity = #entity;

            fn get_indicies(&mut self, entity: &whim::tables::Entry<Self::Entity>) -> Vec<whim::indices::Index> {
                #function
                #return_statement
            }
        }
    }
    .into()
}
