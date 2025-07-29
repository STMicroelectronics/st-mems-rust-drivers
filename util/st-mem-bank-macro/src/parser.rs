use quote::quote;
use syn::{
    parse::Parser, punctuated::Punctuated, Attribute, Expr, Fields, ItemStruct, Meta, Token, Type
};

pub fn extract_value_from_attribute(attr: &Attribute) -> Option<usize> {
    let mut value = None;
    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        let parser = Punctuated::<Expr, Token![,]>::parse_terminated;

        if let Ok(exprs) =  parser.parse2(meta_list.tokens.clone()) {
            for expr in exprs.iter() {
                if let Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit_int), .. }) = expr {
                    value = lit_int.base10_parse::<usize>().ok();
                }
            }
        }
    }

    value
}


/// Analize the struct and summarize information about primitive types and
/// when used as array. Extract also information about `padding_before` and
/// `padding_after`
///
/// # Returns
///
/// * `Type`: it could be a primitive type or complex type such array
/// * `array_size`: If the struct is defined as array otherwise returns 1
/// * `padding_before`: number of bytes to shift the inner content
/// * `padding_after`: number of bytes to shift the inner content
pub fn get_type_and_array_size(item: &mut ItemStruct) -> (Type, usize, u8, u8) {

    match &mut item.fields {
        Fields::Unnamed(fields_unnamed) if fields_unnamed.unnamed.len() == 1 => {
            // Expect type like [DataType; N]
            if let Type::Array(type_array) = &fields_unnamed.unnamed[0].ty {
                let elem_type = &*type_array.elem;
                let len_expr = &type_array.len;

                // Try to parse length as usize literal
                let n_array = if let syn::Expr::Lit(expr_lit) = len_expr {
                    if let syn::Lit::Int(lit_int) = &expr_lit.lit {
                        lit_int.base10_parse::<usize>().unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };

                (elem_type.clone(), n_array, 0, 0)
            } else {
                // Not an array type
                (fields_unnamed.unnamed[0].ty.clone(), 1, 0, 0)
            }
        },
        Fields::Named(fields_named) => {
            // Handle the bitfield-structs
            let mut total_bits: u8 = 0;
            let mut offset_before: u8 = 0;
            let mut offset_after: u8 = 0;

            for field in &mut fields_named.named {

                let mut bits_value = None;

                // find the offset_before and offset_after tag
                for attr in &field.attrs {
                    if attr.path().is_ident("bits") {
                        bits_value = extract_value_from_attribute(attr);
                    } else if attr.path().is_ident("offset_before") {
                        offset_before = extract_value_from_attribute(attr).unwrap_or(0) as u8;
                    } else if attr.path().is_ident("offset_after") {
                        offset_after = extract_value_from_attribute(attr).unwrap_or(0) as u8;
                    }
                }

                // remove the offset_before and offset_after tag
                let bits_value = bits_value.unwrap_or(0) as u8;
                field.attrs.retain(|attr| !attr.path().is_ident("offset_after"));
                field.attrs.retain(|attr| !attr.path().is_ident("offset_before"));

                if offset_after % 8 != 0 || offset_before % 8 != 0 {
                    panic!("offset should have bits multiple of bytes");
                }

                total_bits += bits_value;
            }


            let reg_type = match total_bits {
                8 => "u8",
                16 => "u16",
                32 => "u32",
                _ => "u8"
            };

            (syn::parse_str(reg_type).unwrap(), 1, offset_before / 8, offset_after / 8)
        }
        _ => {
            // For other struct types, fallback
            (syn::parse_str("u8").unwrap(), 1, 0, 0)
        }
    }
}

pub(crate) enum Lifetime {
    A,
    Anonym,
    None
}


/// It creates the generics part based on the input
pub fn generate_generics(lifetime: Lifetime, generics_num: u8) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {

    let generics_str = match lifetime {
        Lifetime::A => quote! { <'a, },
        Lifetime::Anonym => quote! { <'_, },
        Lifetime::None => quote! { < },
    };

    let (timer_short, timer_long) = if generics_num == 2 {
        (quote! { T }, quote! { T: DelayNs })
    } else {
        (quote! {}, quote! {})
    };

    let gen_short = quote! {#generics_str B, #timer_short >};
    let gen_long = quote! {#generics_str B: BusOperation, #timer_long >};

    (gen_long, gen_short)
}
