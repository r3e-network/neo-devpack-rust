use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, LitStr, Token};

/// Parser for permission arguments: contract and method list.
pub(crate) struct PermissionArgs {
    pub(crate) contract: LitStr,
    pub(crate) methods: Vec<LitStr>,
}

impl Parse for PermissionArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let contract: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let content;
        bracketed!(content in input);
        let methods: Punctuated<LitStr, Token![,]> =
            content.call(Punctuated::<LitStr, Token![,]>::parse_terminated)?;

        Ok(Self {
            contract,
            methods: methods.into_iter().collect(),
        })
    }
}

/// Parser for a bracketed list of string literals.
pub(crate) struct StringList {
    pub(crate) items: Vec<LitStr>,
}

impl Parse for StringList {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        let items: Punctuated<LitStr, Token![,]> =
            content.call(Punctuated::<LitStr, Token![,]>::parse_terminated)?;
        Ok(Self {
            items: items.into_iter().collect(),
        })
    }
}

