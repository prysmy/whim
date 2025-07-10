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
    index_type: syn::Type,
    entity_type: syn::Type,
}

impl syn::parse::Parse for IndexerAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let index_type: syn::Type = input.parse()?;
        input.parse::<syn::Token![->]>()?; // Expect a comma after the
        let entity_type: syn::Type = input.parse()?;
        Ok(IndexerAttr {
            index_type,
            entity_type,
        })
    }
}

#[allow(clippy::cmp_owned)]
#[proc_macro_attribute]
pub fn index(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let IndexerAttr {
        index_type,
        entity_type,
    } = syn::parse_macro_input!(attrs as IndexerAttr);

    let attributes = &function.attrs;
    let vis = &function.vis;
    let struct_name = &function.sig.ident;
    let generator_input = &function.sig.inputs;
    let generator_output = &function.sig.output;
    let generator_block = &function.block;

    let return_type = match &function.sig.output {
        syn::ReturnType::Default => {
            return syn::Error::new_spanned(function.sig, "Indexer function must return a type")
                .to_compile_error()
                .into();
        }
        syn::ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
    };

    let return_conversion = if return_type == format!("Vec < {} >", index_type.to_token_stream()) {
        quote! {
            generator(entity)
        }
    } else if return_type == format!("Option < {} >", index_type.to_token_stream()) {
        quote! {
            generator(entity).into_iter().collect::<Vec<_>>()
        }
    } else if return_type == index_type.to_token_stream().to_string() {
        quote! {
            vec![generator(entity)]
        }
    } else {
        return syn::Error::new_spanned(
            function.sig,
            "Indexer function must return IndexType, Vec<IndexType> or Option<IndexType>",
        )
        .to_compile_error()
        .into();
    };

    quote! {
        #(#attributes)*
        #vis struct #struct_name {
            storage: whim::indices::IndexStorage<#index_type, #entity_type>,
        }

        impl #struct_name {
            fn generate_indicies(
                &self,
                entity: &whim::tables::Entry<#entity_type>,
            ) -> Vec<#index_type> {
                fn generator(#generator_input) #generator_output #generator_block

                #return_conversion
            }

            pub fn find(
                &self,
                key: &#index_type,
            ) -> Vec<&whim::tables::Entry<#entity_type>> {
                self.storage.get(key)
            }
        }

        impl whim::indices::Indexer for #struct_name {
            type Entity = #entity_type;

            fn index(&mut self, entity: &Entry<Self::Entity>) {
                let keys = self.generate_indicies(entity);
                self.storage.push(keys, entity);
            }

            fn forget(&mut self, entity: &Entry<Self::Entity>) {
                let keys = self.generate_indicies(entity);
                self.storage.forget(keys, entity);
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    storage: whim::indices::IndexStorage::default(),
                }
            }
        }
    }
    .into()
}
