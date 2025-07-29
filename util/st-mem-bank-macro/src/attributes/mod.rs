use proc_macro2::TokenStream;

pub mod register;
pub mod adv_register;
pub mod struct_register;
pub mod named_register;
pub mod mem_bank;

use quote::quote;


#[derive(Clone, Copy, PartialEq, Eq)]
pub (crate) enum Order {
    Forward,
    Inverse
}

impl Order {
    pub fn from_x_bytes_word(&self) -> TokenStream {
        if *self == Order::Forward {
            quote! {from_le_bytes}
        } else {
            quote! {from_be_bytes}
        }
    }

    pub fn to_x_bytes_word(&self) -> TokenStream {
        if *self == Order::Forward {
            quote! {to_le_bytes}
        } else {
            quote! {to_be_bytes}
        }
    }
}
