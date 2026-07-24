extern crate proc_macro;
use proc_macro::TokenStream;
use qr::LogoPixel;
use quote::{ToTokens, quote};
use syn::{
    Ident, LitStr, Token, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token,
};

mod generator;

use generator::generate_qr_code_with_logo;

use crate::generator::QrGeneratorError;

enum QrLogo {
    HasLogo {
        _plus_token: token::Plus,
        _bracket_token: token::Bracket,
        logo: Punctuated<LitStr, token::Comma>,
    },
    None,
}

struct QrEnvFallback {
    _or: Token![||],
    val: LitStr,
}

enum QrContent {
    Env {
        _env: Ident,
        _excl: Token![!],
        _paren_token: token::Paren,
        var: LitStr,
        fallback: Option<QrEnvFallback>,
    },
    Raw(LitStr),
}

struct QrInput {
    const_token: Token![const],
    ident: Ident,
    _equals_token: token::Eq,
    input: QrContent,
    logo: QrLogo,
    end_token: Option<token::Semi>,
}

impl Parse for QrLogo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(token::Plus) {
            let logo;
            Ok(QrLogo::HasLogo {
                _plus_token: input.parse()?,
                _bracket_token: bracketed!(logo in input),
                logo: logo.parse_terminated(<LitStr as Parse>::parse, token::Comma)?,
            })
        } else {
            Ok(QrLogo::None)
        }
    }
}

impl Parse for QrEnvFallback {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(QrEnvFallback {
            _or: input.parse()?,
            val: input.parse()?,
        })
    }
}

impl Parse for QrContent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) {
            let env: Ident = input.parse()?;
            if env != "env" {
                return Err(syn::Error::new(
                    env.span(),
                    format!("Invalid identifier {env}"),
                ));
            }
            let var;
            let mut result = QrContent::Env {
                _env: env,
                _excl: input.parse()?,
                _paren_token: parenthesized!(var in input),
                var: var.parse()?,
                fallback: None,
            };
            if input.fork().parse::<QrEnvFallback>().is_ok() {
                let QrContent::Env { fallback, .. } = &mut result else {
                    unreachable!()
                };
                *fallback = Some(input.parse()?);
            }
            Ok(result)
        } else {
            Ok(QrContent::Raw(input.parse()?))
        }
    }
}

impl Parse for QrInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(QrInput {
            const_token: input.parse()?,
            ident: input.parse()?,
            _equals_token: input.parse()?,
            input: input.parse()?,
            logo: input.parse()?,
            end_token: input.parse()?,
        })
    }
}

#[derive(thiserror::Error, Debug)]
enum QrMacroError {
    #[error(transparent)]
    Logo(#[from] LogoParseError),
    #[error(transparent)]
    Env(#[from] std::env::VarError),
    #[error(transparent)]
    Qr(#[from] QrGeneratorError),
}

#[derive(thiserror::Error, Debug)]
enum LogoParseError {
    #[error("Invalid pixel data")]
    InvalidPixel,
}

fn into_logo(lines: impl Iterator<Item = LitStr>) -> Result<Vec<Vec<LogoPixel>>, LogoParseError> {
    lines
        .into_iter()
        .map(|line| {
            line.value()
                .chars()
                .map(|x| match x {
                    'x' => Ok(LogoPixel::Dark),
                    'o' => Ok(LogoPixel::Light),
                    '/' => Ok(LogoPixel::Transparent),
                    _err => Err(LogoParseError::InvalidPixel),
                })
                .collect()
        })
        .collect()
}

fn print_error(d: impl std::fmt::Display) -> TokenStream {
    let f = format!("Error: {d}");
    quote! { compile_error!(#f); }.into()
}

#[proc_macro]
pub fn make_qr(input: TokenStream) -> TokenStream {
    match make_qr_inner(parse_macro_input!(input as QrInput)) {
        Ok(output) => output,
        Err(e) => print_error(e),
    }
}
fn make_qr_inner(input: QrInput) -> Result<TokenStream, QrMacroError> {
    let QrInput {
        const_token,
        ident,
        _equals_token,
        input,
        logo,
        end_token,
    } = input;
    let logo = match logo {
        QrLogo::HasLogo { logo, .. } => into_logo(logo.into_iter())?,
        QrLogo::None => vec![],
    };
    let input = match input {
        QrContent::Env {
            var,
            fallback: None,
            ..
        } => std::env::var(var.value())?,
        QrContent::Env {
            var,
            fallback: Some(fallback),
            ..
        } => std::env::var(var.value()).unwrap_or(fallback.val.value()),
        QrContent::Raw(lit_str) => lit_str.value(),
    };
    let res = generate_qr_code_with_logo(&input, logo.into_iter().map(|x| x.into_iter()), None)?;
    let width = res.width;
    let height = res.code.len() / width;
    let [logo_offset_x, logo_offset_y] = res.logo_offset;
    let inverts = res.logo_inverts;
    let i_width = res.logo_width;
    let i_height = inverts.len().checked_div(i_width).unwrap_or(0);
    let logo_data = {
        let mut logo_data = Default::default();
        res.code.chunks(width).for_each(|c| {
            let mut s = Default::default();
            c.iter().for_each(|x| quote! { #x, }.to_tokens(&mut s));
            quote! { [ #s ], }.to_tokens(&mut logo_data);
        });
        logo_data
    };
    let inv_data = {
        let mut inv_data = Default::default();
        if i_width > 0 {
            inverts.chunks(i_width).for_each(|c| {
                let mut s = Default::default();
                c.iter().for_each(|x| quote! { #x, }.to_tokens(&mut s));
                quote! { [ #s ], }.to_tokens(&mut inv_data);
            });
        }
        inv_data
    };

    let qr_type = quote! { qr::QrCodeWithLogo };
    let res = quote! {
        #const_token #ident: #qr_type<#width, #height, #logo_offset_x, #logo_offset_y, #i_width, #i_height> = #qr_type {
            code: [
                #logo_data
            ],
            inverted: [
                #inv_data
            ],
        } #end_token
    };
    Ok(res.into())
}
