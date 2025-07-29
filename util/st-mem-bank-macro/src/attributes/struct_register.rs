use syn::Field;
use quote::format_ident;
use proc_macro2::TokenStream;
use syn::{
    Type, TypePath, Ident,
};

use quote::quote;

pub(crate) struct StructRegisterAttr {}

impl StructRegisterAttr {
    pub fn size_of_type(ty: &Type) -> Option<usize> {
        if let Type::Path(TypePath { qself: None, path }) = ty {
            if path.segments.len() == 1 {
                let ident = &path.segments[0].ident;
                return match ident.to_string().as_str() {
                    "u8" | "i8" => Some(1),
                    "u16" | "i16" => Some(2),
                    "u32" | "i32" => Some(4),
                    "u64" | "i64" => Some(8),
                    "u128" | "i128" => Some(16),
                    _ => None,
                };
            }
        }
        None
    }

    pub fn generate_to_le_bytes(fields: &[Field], buffer_size: &TokenStream) -> TokenStream {
        // Collect (field_name, size) pairs
        let names_sizes: Vec<(Ident, usize)> = fields.iter().map(|field| {
            let name = field.ident.clone().expect("Named field should have name");
            let size = Self::size_of_type(&field.ty).expect("Only primitive types are allowed in named structs");
            (name, size)
        }).collect();

        // Generate let bindings: let _field = self.field.to_le_bytes();
        let let_bindings = names_sizes.iter().map(|(name, _size)| {
            let tmp_name = format_ident!("_{}", name);
            quote! {
                let #tmp_name = self.#name.to_le_bytes();
            }
        });

        // Generate the concatenated byte array by flattening all bytes from each _field
        // For each field, generate _field[0], _field[1], ..., _field[size-1]
        let expanded_bytes = names_sizes.iter().flat_map(|(name, size)| {
            let tmp_name = format_ident!("_{}", name);
            (0..*size).map(move |i| {
                let index = syn::Index::from(i);
                quote! { #tmp_name[#index] }
            })
        });

        // Generate the final to_le_bytes function
        quote! {
            pub fn to_le_bytes(&self) -> [u8; #buffer_size] {
                #(#let_bindings)*

                [
                    #(#expanded_bytes),*
                ]
            }
        }
    }

    pub fn generate_from_le_bytes(fields: &Vec<Field>, buffer_size: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let mut offset = 0usize;
        let indexed_fields = fields.iter().map(|field| {
            let name = field.ident.clone().expect("Named field should have name");
            let size = Self::size_of_type(&field.ty).expect("Only primitive types are allowed in named structs");
            let ty = &field.ty;

            let start = offset;
            offset += size;

            // Generate array of bytes: [bytes[start], bytes[start+1], ..., bytes[start+size-1]]
            let indices = (start..start + size).map(|i| {
                quote! { bytes[#i] }
            });

            quote! {
                #name: <#ty>::from_le_bytes([#(#indices),*])
            }
        });

        quote! {
            pub fn from_le_bytes(bytes: [u8; #buffer_size]) -> Self {
                Self {
                    #(#indexed_fields),*
                }
            }
        }
    }
}
