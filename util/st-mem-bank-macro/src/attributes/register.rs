use syn::punctuated::Punctuated;
use syn::{
    Expr, Path, Result, Token, Type, TypePath, ExprPath, Lit,
    parse::{Parse, ParseStream},
};

use quote::quote;
use crate::attributes::Order;
use crate::generator::QuoteOutput;

pub(crate) struct RegisterAttr {
    pub address: Path,
    pub access_type: Path,
    pub init_fn: Option<Path>,
    pub override_type: Option<Type>,
    pub generics_num: u8,
    pub order: Order
}

impl Parse for RegisterAttr {
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

        Ok(RegisterAttr {
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

impl QuoteOutput for RegisterAttr {
    fn quote_read(&self) -> proc_macro2::TokenStream {
        let address = &self.address;
        quote! { sensor.read_from_register(#address as u8, buff) }
    }
    
    fn quote_write_single(&self) -> proc_macro2::TokenStream {
        let address = &self.address;
        quote! { sensor.write_to_register(#address as u8, &[self.0]) }
    }
    
    fn quote_write_multi(&self) -> proc_macro2::TokenStream {
        let address = &self.address;
        let to_fn = self.order.to_x_bytes_word();
        quote! { sensor.write_to_register(#address as u8, &self.0.#to_fn()) }
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
