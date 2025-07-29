use proc_macro::TokenStream;
use syn::{punctuated::Punctuated, DeriveInput, Field};
use syn::{
    Expr, Path, Result, Token, Type, TypePath, ExprPath, Lit, Fields,
    parse::{Parse, ParseStream},
    parse_macro_input
};

use crate::attributes::struct_register::StructRegisterAttr;
use quote::quote;
use crate::attributes::Order;
use crate::generator::QuoteOutput;

pub(crate) struct NamedRegisterAttr {
    pub address: Path,
    pub access_type: Path,
    pub init_fn: Option<Path>,
    pub override_type: Option<Type>,
    pub generics_num: u8,
    pub order: Order
}

impl Parse for NamedRegisterAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut access_type = None;
        let mut address = None;
        let mut init_fn = None;
        let mut override_type = None;
        let mut order = Order::Forward;
        let mut generics_num = None;

        // Parse comma-separated key-value pairs
        let pairs = Punctuated::<syn::MetaNameValue, Token![,]>::parse_terminated(input)?;

        for pair in pairs {
            let ident = pair.path.get_ident().unwrap().to_string();
            match ident.as_str() {
                "address" => {
                    if let Expr::Path(path) = &pair.value {
                        address = Some(path.path.clone());
                    }
                }
                "access_type" => {
                    if let Expr::Path(path) = &pair.value {
                        access_type = Some(path.path.clone());
                    }
                },
                "generics" => {
                    if let Expr::Lit(expr_lit) = &pair.value {
                        if let Lit::Int(lit_int) = &expr_lit.lit {
                             generics_num = lit_int.base10_parse::<u8>().ok();
                        }
                    }
                },
                "init_fn" => {
                    if let Expr::Path(path) = &pair.value {
                        init_fn = Some(path.path.clone());
                    }
                },
                "override_type" => {
                    if let Expr::Path(path) = &pair.value {
                        override_type = Some(expr_path_to_type(&path));
                    }
                },
                "order" => {
                    if let Expr::Path(path) = &pair.value {
                        if path.path.segments[0].ident == "Inverse" {
                            order = Order::Inverse;
                        } else {
                            println!("`{}` is not valid for the order attribute", path.path.segments[0].ident);
                        }
                    }
                }
                _ => {}
            }
        }

        let access_type = access_type.ok_or_else(|| input.error("missing 'state' argument"))?;
        let address = address.ok_or_else(|| input.error("missing 'address' argument"))?;
        let generics_num = generics_num.ok_or_else(|| input.error("missing 'generic' argument"))?;

        Ok(NamedRegisterAttr {
            access_type,
            address,
            init_fn,
            override_type,
            generics_num,
            order
        })
    }
}

fn expr_path_to_type(expr: &ExprPath) -> Type {
    let type_path = TypePath {
        qself: None,
        path: expr.path.clone(),
    };
    Type::Path(type_path)
}

impl QuoteOutput for NamedRegisterAttr {
    fn quote_read(&self) -> proc_macro2::TokenStream {
        let address = &self.address;
        quote! { sensor.read_from_register(#address as u8, buff) }
    }
    
    fn quote_write_single(&self) -> proc_macro2::TokenStream {
        //let address = &self.address;
        //quote! { sensor.write_to_register(#address as u8, &[self]) }
        panic!("not required");
    }
    
    fn quote_write_multi(&self) -> proc_macro2::TokenStream {
        let address = &self.address;
        let to_fn = self.order.to_x_bytes_word();
        quote! { sensor.write_to_register(#address as u8, &self.#to_fn()) }
    }

    fn quote_write_to_buff(&self) -> proc_macro2::TokenStream {
        let address = &self.address;
        quote! { sensor.write_to_register(#address as u8, &buff) }
    }

    fn get_access_type(&self) -> &Path {
        &self.access_type
    }

    fn get_init(&self) -> proc_macro2::TokenStream {
        if let Some(init_fn) = &self.init_fn {
            quote! { #init_fn() }
        } else {
            quote! { 0 }
        }
    }

    fn get_override_type(&self) -> Option<Type> {
        self.override_type.clone()
    }

    fn get_order(&self) -> Order {
        self.order
    }

    fn get_generics_num(&self) -> u8 {
        self.generics_num
    }
}

impl NamedRegisterAttr {

    pub fn bytes_number(fields: &Vec<Field>) -> usize {
        let mut total_bytes = 0;
        for f in fields {
            total_bytes += StructRegisterAttr::size_of_type(&f.ty).expect("Only primitive types are allowed");
        }

        total_bytes
    }

    pub fn create_to_from_le_bytes(input: TokenStream, bytes_number: &mut usize) -> TokenStream {
        // Parse the input tokens into a syntax tree
        let input = parse_macro_input!(input as DeriveInput);
        let struct_name = &input.ident;

        let fields = match &input.data {
            syn::Data::Struct(data_struct) => match &data_struct.fields {
                Fields::Named(fields_named) => fields_named.named.iter().cloned().collect::<Vec<_>>(),
                _ => {
                    return syn::Error::new_spanned(
                        &input,
                        "named_register only supports structs with named fields",
                    )
                    .to_compile_error()
                    .into();
                }
            },
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "named_register derive only supports structs",
                )
                .to_compile_error()
                .into();
            }
        };

        *bytes_number = Self::bytes_number(&fields);

        let to_le_bytes = StructRegisterAttr::generate_to_le_bytes(&fields, &quote! { #bytes_number } );
        let from_le_bytes = StructRegisterAttr::generate_from_le_bytes(&fields, &quote! { #bytes_number });


        let expanded = quote! {
            impl #struct_name {
                #to_le_bytes
                #from_le_bytes
            }
        };

        TokenStream::from(expanded)
    }
}
