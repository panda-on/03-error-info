use darling::{
    ast::{Data, Fields, Style},
    util, FromDeriveInput, FromVariant,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct ToErrorInfoReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<ToErrorInfoVariantReceiver, ()>,
    app_type: syn::Type,
    prefix: String,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(error_info))]
struct ToErrorInfoVariantReceiver {
    ident: syn::Ident,
    fields: Fields<util::Ignored>,
    code: String,
    #[darling(default)]
    app_code: String,
    #[darling(default)]
    client_msg: String,
}

pub(crate) fn impl_to_error_info(input: DeriveInput) -> TokenStream {
    let ToErrorInfoReceiver {
        ident: name,
        generics,
        data: Data::Enum(variants),
        app_type,
        prefix,
    } = ToErrorInfoReceiver::from_derive_input(&input).expect("can not parse input")
    else {
        panic!("macro only works on enums");
    };

    let match_arms = variants
        .iter()
        .map(|variant| {
            let ToErrorInfoVariantReceiver {
                ident,
                fields,
                code,
                app_code,
                client_msg,
            } = variant;

            let code = format!("{}{}", prefix, code);

            let variant_code = match fields.style {
                Style::Struct => quote! { #name::#ident { .. }},
                Style::Tuple => quote! { #name::#ident(_)},
                Style::Unit => quote! { #name::#ident },
            };

            quote! {
                #variant_code => {
                    ErrorInfo::new(
                        #app_code,
                        #code,
                        #client_msg,
                        self,
                    )
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        use error_code::{ErrorInfo, ToErrorInfo as _};
        impl #generics ToErrorInfo for #name #generics {
            type T = #app_type;

            fn to_error_info(&self) -> ErrorInfo<Self::T> {
                match self {
                    #(#match_arms),*
                }
            }
        }

    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_input() {
        let input = r#"
        #[derive(Debug, thiserror::Error, ToErrorInfo)]
        #[error_info(app_type = "http::StatusCode", prefix = "01")]
        pub enum MyError {
            #[error("Invalid command: {0}")]
            #[error_info(code = "IC", app_code = "400")]
            InvalidCommand(String),

            #[error("Internal argument: {0}")]
            #[error_info(code = "IA", app_code = "400", client_msg = "friendly msg")]
            InternalArgument(String),

            #[error("{0}")]
            #[error_info(code = "RE", app_code = "500")]
            RespError(#[from] std::io::Error),
        }"#;

        let ast = syn::parse_str(input).unwrap();
        let info = ToErrorInfoReceiver::from_derive_input(&ast).unwrap();
        assert_eq!(info.ident.to_string(), "MyError");
        assert_eq!(info.prefix, "01");
        let code = impl_to_error_info(ast);
        println!("code: {}", code);
    }
}
