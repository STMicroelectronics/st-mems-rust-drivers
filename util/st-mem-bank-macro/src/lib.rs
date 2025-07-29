extern crate proc_macro;

mod attributes;
mod generator;
mod parser;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, Fields, ItemStruct, Path, PathSegment, Type, TypePath, Ident
};
use generator::{Quote, QuoteOutput};
use attributes::{adv_register::AdvRegisterAttr, mem_bank::{MemBank, MemBankArgs}, named_register::NamedRegisterAttr, register::RegisterAttr, struct_register::StructRegisterAttr};
use parser::*;



#[proc_macro_derive(MultiRegister, attributes(struct_register_attr))]
pub fn multi_register_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named.named.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "MultiRegister derive only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input,
                "MultiRegister derive only supports structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let type_path = TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments: std::iter::once(PathSegment::from(struct_name.clone())).collect(),
        },
    };
    let data_type = Type::Path(type_path);

    let buffer_size = quote! { core::mem::size_of::<#data_type>() };

    let to_le_bytes = StructRegisterAttr::generate_to_le_bytes(&fields, &buffer_size);
    let from_le_bytes = StructRegisterAttr::generate_from_le_bytes(&fields, &buffer_size);

    let expanded = quote! {
        impl #struct_name {
            #to_le_bytes
            #from_le_bytes
        }
    };

    TokenStream::from(expanded)
}


#[proc_macro_attribute]
pub fn adv_register(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let args = parse_macro_input!(attr as AdvRegisterAttr);

    let (data_type, n_array, offset_before, offset_after) = get_type_and_array_size(&mut input); 

    let generics = generate_generics(Lifetime::None, args.get_generics_num());

    let mut quote = Quote::new(
        &input,
        args,
        &generics,
        &data_type,
        n_array,
        false,
        offset_before,
        offset_after
    );

    let expanded = quote.generate();

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn register(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let args = parse_macro_input!(attr as RegisterAttr);

    let (data_type, n_array, offset_before, offset_after) = get_type_and_array_size(&mut input); 
    let generics = generate_generics(Lifetime::None, args.get_generics_num());

    let mut quote = Quote::new(
        &input,
        args,
        &generics,
        &data_type,
        n_array,
        false,
        offset_before,
        offset_after
    );

    let expanded = quote.generate();

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn named_register(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item2 = item.clone();
    let input = parse_macro_input!(item2 as ItemStruct);
    let args = parse_macro_input!(attr as NamedRegisterAttr);

    let mut bytes: usize = 0;
    let to_from_le_bytes: proc_macro2::TokenStream = NamedRegisterAttr::create_to_from_le_bytes(item, &mut bytes).into(); 


    let struct_ident: &Ident = &input.ident;
    let path_segment = PathSegment::from(struct_ident.clone());
    let path = Path {
        leading_colon: None,
        segments: std::iter::once(path_segment).collect(),
    };
    let type_path = TypePath { qself: None, path };
    let data_type = Type::Path(type_path);


    let generics = generate_generics(Lifetime::None, args.get_generics_num());
    let mut quote = Quote::new(
        &input,
        args,
        &generics,
        &data_type,
        1,
        true,
        0,
        0
    );

    quote.override_buffer_size(bytes);

    let expanded = quote.generate();

    TokenStream::from(quote! {
        #expanded
        #to_from_le_bytes
    })
}

#[proc_macro_attribute]
pub fn mem_bank(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as MemBank);
    let sensor = parse_macro_input!(attr as MemBankArgs);

    TokenStream::from(input.create_output(&sensor)) 
}
