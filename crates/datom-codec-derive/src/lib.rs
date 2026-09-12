use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

fn fields<'a>(input: &'a DeriveInput, capability: &str) -> Result<&'a Fields, TokenStream> {
    match &input.data {
        Data::Struct(data) => Ok(&data.fields),
        _ => Err(
            syn::Error::new_spanned(&input.ident, format!("{capability} supports structs"))
                .into_compile_error()
                .into(),
        ),
    }
}

#[proc_macro_derive(Compositional)]
pub fn compositional(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    if let Data::Enum(data) = &input.data {
        let mut bounded_generics = input.generics.clone();
        for parameter in &input.generics.params {
            if let syn::GenericParam::Type(parameter) = parameter {
                let ident = &parameter.ident;
                bounded_generics
                    .make_where_clause()
                    .predicates
                    .push(syn::parse_quote!(#ident: ::datom_codec::Compositional));
            }
        }
        let (impl_generics, ty_generics, where_clause) = bounded_generics.split_for_impl();
        let unit_arms = data.variants.iter().filter_map(|variant| {
            if !variant.fields.is_empty() {
                return None;
            }
            let build = match variant.fields {
                Fields::Named(_) => quote!({}),
                Fields::Unnamed(_) => quote!(()),
                Fields::Unit => quote!(),
            };
            let spelling = variant.ident.to_string();
            let name = &variant.ident;
            Some(quote!(#spelling => Ok(Self::#name #build)))
        });
        let arms = data.variants.iter().filter(|variant| !variant.fields.is_empty()).map(|variant| {
            let variant_name = &variant.ident;
            let spelling = variant_name.to_string();
            let fields = &variant.fields;
            let reads = fields.iter().enumerate().map(|(index, field)| { let ty = &field.ty; let binding = syn::Ident::new(&format!("field_{index}"), proc_macro2::Span::call_site()); quote!(let #binding: #ty = positions.position(budget)?;) });
            let bindings: Vec<_> = (0..fields.len()).map(|index| syn::Ident::new(&format!("field_{index}"), proc_macro2::Span::call_site())).collect();
            let build = match fields { Fields::Named(named) => { let names = named.named.iter().map(|field| field.ident.as_ref().unwrap()); quote!(Self::#variant_name { #(#names: #bindings),* }) }, Fields::Unnamed(_) => quote!(Self::#variant_name(#(#bindings),*)), Fields::Unit => quote!(Self::#variant_name) };
            let count = fields.len();
            if count == 1 {
                let binding = &bindings[0];
                quote!(#spelling => { let #binding = body.compose(budget)?; Ok(#build) })
            } else {
quote!(#spelling => { let mut positions = ::datom_codec::DatomPositioning::positions(body, #count as ::datom_codec::Integer)?; #(#reads)* Ok(#build) })
            }
        });
        return quote! {
            impl #impl_generics ::datom_codec::Compositional for #name #ty_generics #where_clause {
                fn compose(datom: &::datom_codec::Datom, budget: &mut ::datom_codec::Budget) -> Result<Self, ::datom_codec::Error> {
                    use ::datom_codec::{Budgeting, Composable, Positioning, Variantizing};
                    match &datom.form {
                        ::datom_codec::Form::Bare(head) => { budget.spend(&datom.path)?; match head.as_str() { #(#unit_arms,)* other => Err(<::datom_codec::Error as ::datom_codec::ErrorRaising>::composition(datom.path.clone(), ::datom_codec::ErrorKind::Variant { expected: stringify!(#name).to_owned(), found: other.to_owned() })) } },
                        _ => { let (head, body) = datom.variant(budget, "Variant")?; match head { #(#arms,)* other => Err(<::datom_codec::Error as ::datom_codec::ErrorRaising>::composition(datom.path.clone(), ::datom_codec::ErrorKind::Variant { expected: stringify!(#name).to_owned(), found: other.to_owned() })) } }
                    }
                }
            }
        }.into();
    }
    let fields = match fields(&input, "Compositional") {
        Ok(fields) => fields,
        Err(error) => return error,
    };
    let mut bounded_generics = input.generics.clone();
    for parameter in &input.generics.params {
        if let syn::GenericParam::Type(parameter) = parameter {
            let ident = &parameter.ident;
            bounded_generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ident: ::datom_codec::Compositional));
        }
    }
    let (impl_generics, ty_generics, where_clause) = bounded_generics.split_for_impl();
    let reads = fields.iter().enumerate().map(|(index, field)| {
        let ty = &field.ty;
        let binding = syn::Ident::new(&format!("field_{index}"), proc_macro2::Span::call_site());
        quote!(let #binding: #ty = positions.position(budget)?;)
    });
    let bindings: Vec<_> = (0..fields.len())
        .map(|index| syn::Ident::new(&format!("field_{index}"), proc_macro2::Span::call_site()))
        .collect();
    let build = match fields {
        Fields::Named(named) => {
            let names = named
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap());
            quote!(Self { #(#names: #bindings),* })
        }
        Fields::Unnamed(_) => quote!(Self(#(#bindings),*)),
        Fields::Unit => quote!(Self),
    };
    let arity = fields.len();
    quote! {
        impl #impl_generics ::datom_codec::Compositional for #name #ty_generics #where_clause {
            fn compose(datom: &::datom_codec::Datom, budget: &mut ::datom_codec::Budget) -> Result<Self, ::datom_codec::Error> {
                use ::datom_codec::{Budgeting, DatomPositioning, Positioning};
                budget.spend(&datom.path)?;
                let mut positions = datom.positions(#arity as ::datom_codec::Integer)?;
                #(#reads)* Ok(#build)
            }
        }
    }.into()
}

#[proc_macro_derive(Datomizable)]
pub fn datomizable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    if let Data::Enum(data) = &input.data {
        let mut bounded_generics = input.generics.clone();
        for parameter in &input.generics.params {
            if let syn::GenericParam::Type(parameter) = parameter {
                let ident = &parameter.ident;
                bounded_generics.make_where_clause().predicates.push(
                    syn::parse_quote!(#ident: ::datom_codec::Datomizable<Output = ::datom_codec::Datom>),
                );
            }
        }
        let (impl_generics, ty_generics, where_clause) = bounded_generics.split_for_impl();
        let arms = data.variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            let spelling = variant_name.to_string();
            let bindings: Vec<_> = (0..variant.fields.len()).map(|index| syn::Ident::new(&format!("field_{index}"), proc_macro2::Span::call_site())).collect();
            let pattern = match &variant.fields { Fields::Named(named) => { let names = named.named.iter().map(|field| field.ident.as_ref().unwrap()); quote!(Self::#variant_name { #(#names: #bindings),* }) }, Fields::Unnamed(_) => quote!(Self::#variant_name(#(#bindings),*)), Fields::Unit => quote!(Self::#variant_name) };
            let child_values: Vec<_> = bindings.iter().enumerate().map(|(index, binding)| quote!((#binding).datomize(at.child(1).child(#index as ::datom_codec::Integer)))).collect();
            let body = match variant.fields.len() {
                1 => { let binding = &bindings[0]; quote!((#binding).datomize(at.child(1))) },
                _ => quote!(::datom_codec::Datom { path: at.child(1), form: ::datom_codec::Form::Struct(vec![#(#child_values),*]) }),
            };
            if variant.fields.is_empty() {
                quote!(#pattern => ::datom_codec::Datom { path: at.clone(), form: ::datom_codec::Form::Bare(#spelling.to_owned()) })
            } else {
                quote!(#pattern => ::datom_codec::Datom { path: at.clone(), form: ::datom_codec::Form::Variant(::datom_codec::Symbol(#spelling.to_owned()), Box::new(#body)) })
            }
        });
        return quote! {
            impl #impl_generics ::datom_codec::Datomizable for #name #ty_generics #where_clause {
                type Output = ::datom_codec::Datom;
                fn datomize(&self, at: ::datom_codec::Path) -> ::datom_codec::Datom {
                    use ::datom_codec::{Datomizable, Pathing};
                    match self { #(#arms),* }
                }
            }
        }
        .into();
    }
    let fields = match fields(&input, "Datomizable") {
        Ok(fields) => fields,
        Err(error) => return error,
    };
    let mut bounded_generics = input.generics.clone();
    for parameter in &input.generics.params {
        if let syn::GenericParam::Type(parameter) = parameter {
            let ident = &parameter.ident;
            bounded_generics.make_where_clause().predicates.push(
                syn::parse_quote!(#ident: ::datom_codec::Datomizable<Output = ::datom_codec::Datom>),
            );
        }
    }
    let (impl_generics, ty_generics, where_clause) = bounded_generics.split_for_impl();
    let values: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let value = match &field.ident {
                Some(name) => quote!(&self.#name),
                None => {
                    let index = syn::Index::from(index);
                    quote!(&self.#index)
                }
            };
            quote!((#value).datomize(at.child(#index as ::datom_codec::Integer)))
        })
        .collect();
    quote! {
        impl #impl_generics ::datom_codec::Datomizable for #name #ty_generics #where_clause {
            type Output = ::datom_codec::Datom;
            fn datomize(&self, at: ::datom_codec::Path) -> ::datom_codec::Datom {
                use ::datom_codec::{Datomizable, Pathing};
                ::datom_codec::Datom { path: at.clone(), form: ::datom_codec::Form::Struct(vec![#(#values),*]) }
            }
        }
    }.into()
}
