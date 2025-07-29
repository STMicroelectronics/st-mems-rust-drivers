use syn::{
    Path, Type, ItemStruct, PathSegment, TypePath
};

use quote::quote;

use crate::{attributes::Order, StructRegisterAttr};

pub(crate) trait QuoteOutput {
    fn quote_write_single(&self) -> proc_macro2::TokenStream;
    fn quote_write_multi(&self) -> proc_macro2::TokenStream;
    fn quote_write_to_buff(&self) -> proc_macro2::TokenStream;
    fn get_access_type(&self) -> &Path;
    fn quote_read(&self) -> proc_macro2::TokenStream;
    fn get_init(&self) -> proc_macro2::TokenStream;
    fn get_override_type(&self) -> Option<Type>;
    fn get_order(&self) -> Order;
    fn get_generics_num(&self) -> u8;
}

pub(crate) struct Quote<'a, T> where T: QuoteOutput {
    input: &'a ItemStruct,
    args: T, 
    generics: &'a (proc_macro2::TokenStream, proc_macro2::TokenStream),
    data_type: Type,
    pub(crate) buffer_size: proc_macro2::TokenStream,
    n_array: usize,
    use_new: bool,
    offset: (u8, u8)
}

impl<'a, T> Quote<'a, T> where T: QuoteOutput {
    pub fn new(
        input: &'a ItemStruct,
        args: T,
        generics: &'a (proc_macro2::TokenStream, proc_macro2::TokenStream),
        data_type: &'a Type,
        n_array: usize,
        use_new: bool,
        offset_before: u8,
        offset_after: u8
    ) -> Self {
        let mut data_type = data_type.clone();

        if let Some(override_type) = args.get_override_type() {
            data_type = override_type;
        }

        let num_bytes = quote! { core::mem::size_of::<#data_type>() };
        let buffer_size = quote! { #n_array * #num_bytes };

        Self {
            input,
            args,
            generics,
            data_type: data_type.clone(),
            buffer_size,
            n_array,
            use_new,
            offset: (offset_before, offset_after)
        }
    }

    pub fn override_buffer_size(&mut self, size: usize) {
        self.buffer_size = quote! { #size };
    }

    pub fn generate(&mut self) -> proc_macro2::TokenStream {

        if self.n_array > 1 {
            self.quote_for_array()
        } else if Self::is_u8(&self.data_type) {
            self.quote_for_u8()
        } else {
            self.quote_for_multibyte()
        }
    }

    fn is_u8(ty: &Type) -> bool {
        if let Type::Path(TypePath { path, .. }) = ty {
            if let Some(PathSegment { ident, .. }) = path.segments.last() {
                return ident == "u8";
            }
        }
        false
    }

    fn quote_for_u8(&self) -> proc_macro2::TokenStream {
        let (long_generics, short_generics) = self.generics;
        let struct_name = &self.input.ident;

        let Quote {
            input,
            ..
        } = self;

        let read = self.args.quote_read();
        let write = self.args.quote_write_single();
        let access_type = self.args.get_access_type();
        let init = self.args.get_init();

        let create_from_buff = if self.use_new {
            quote! { Self::new(buff[0]) }
        } else {
            quote! { Self(buff[0]) }
        };

        quote! {
            #input

            impl #struct_name {
                pub fn read #long_generics (sensor: &mut #access_type #short_generics) -> Result<Self, Error<B::Error>> {
                    let mut buff = [#init; 1];
                    Self::read_more(sensor, &mut buff)?;
                    Ok(#create_from_buff)
                }

                pub fn write #long_generics (&self, sensor: &mut #access_type #short_generics) -> Result<(), Error<B::Error>> {
                    #write
                }
                
                #[inline]
                pub fn read_more #long_generics (sensor: &mut #access_type #short_generics, buff: &mut [u8]) -> Result<(), Error<B::Error>> {
                    #read
                }
            }
        }
    }

    fn quote_for_multibyte(&self) -> proc_macro2::TokenStream {
        let (long_generics, short_generics) = self.generics;
        let struct_name = &self.input.ident;

        let Quote {
            input,
            data_type,
            buffer_size,
            ..
        } = self;

        let read = self.args.quote_read();
        let write = self.args.quote_write_multi();
        let access_type = self.args.get_access_type();
        let init = self.args.get_init();

        let create_from_val = if self.use_new {
            quote! { val }
        } else {
            quote! { Self(val) }
        };

        let from_fn = self.args.get_order().from_x_bytes_word();

        let read_more = if self.offset.0 + self.offset.1 == 0 {
            quote! { Self::read_more(sensor, &mut buff)?; }
        } else {
            let size = StructRegisterAttr::size_of_type(&self.data_type).expect("Cannot use offset with other than primitive types");
            let offset_before = self.offset.0 as usize;
            let end = size - (self.offset.1 as usize);
            quote! { Self::read_more(sensor, &mut buff[#offset_before..#end])?; }
        };

        quote! {
            #input

            impl #struct_name {
                pub fn read #long_generics (sensor: &mut #access_type #short_generics) -> Result<Self, Error<B::Error>> {
                    let mut buff = [#init; #buffer_size];
                    #read_more
                    let val = <#data_type>::#from_fn(buff);
                    Ok(#create_from_val)
                }

                pub fn write #long_generics (&self, sensor: &mut #access_type #short_generics) -> Result<(), Error<B::Error>> {
                    #write
                }
                
                #[inline]
                pub fn read_more #long_generics (sensor: &mut #access_type #short_generics, buff: &mut [u8]) -> Result<(), Error<B::Error>> {
                    #read
                }
            }
        }
    }

    fn quote_for_array(&self) -> proc_macro2::TokenStream {
        let (long_generics, short_generics) = self.generics;
        let struct_name = &self.input.ident;

        let Quote {
            input,
            data_type,
            n_array,
            ..
        } = self;

        let buffer_size_str = &self.buffer_size;

        let num_bytes = quote! { core::mem::size_of::<#data_type>() };
        let read = self.args.quote_read();
        let write = self.args.quote_write_to_buff();
        let access_type = self.args.get_access_type();
        let init = self.args.get_init();

        // a named register cannot use this function
        
        let from_fn = self.args.get_order().from_x_bytes_word();
        let to_fn = self.args.get_order().to_x_bytes_word();

        let type_size = StructRegisterAttr::size_of_type(&self.data_type);

    
        let type_assignment = if let Some(bytes) = type_size {
            let assignments = (0..*n_array).map(|i| {
                let bytes_str = (0..bytes).map(|j| {
                    let idx = i * bytes + j;    
                    quote! { buff[#idx] }
                });

                quote! {
                    val[#i] = <#data_type>::#from_fn([#(#bytes_str),*]);
                }
            });

            quote! { #(#assignments)* }
        } else {
            quote! {
                for i in 0..#n_array {
                    val[i] = <#data_type>::#from_fn(
                        buff[i * #num_bytes..(i + 1) * #num_bytes]
                            .try_into()
                            .unwrap()
                    );
                }
            }
        };


        quote! {
            #input

            impl #struct_name {
                pub fn read #long_generics (sensor: &mut #access_type #short_generics) -> Result<Self, Error<B::Error>> {
                    let mut buff = [0; #buffer_size_str];
                    Self::read_more(sensor, &mut buff)?;

                    // Process the buffer into the struct
                    let mut val: [#data_type; #n_array] = [#init; #n_array];

                    #type_assignment
                    

                    Ok(Self(val))
                }

                pub fn write #long_generics (&self, sensor: &mut #access_type #short_generics) -> Result<(), Error<B::Error>> {
                    let mut buff = [0; #buffer_size_str];
                    for i in 0..#n_array {
                        buff[i * #num_bytes..(i + 1) * #num_bytes].copy_from_slice(&self.0[i].#to_fn());
                    }

                    #write
                }

                #[inline]
                pub fn read_more #long_generics (sensor: &mut #access_type #short_generics, buff: &mut [u8]) -> Result<(), Error<B::Error>> {
                    #read
                }

            }
        }
    }

}
