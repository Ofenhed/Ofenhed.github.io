use std::{
    fs::{OpenOptions, remove_file},
    io::{ErrorKind, Read as _, Seek as _, SeekFrom},
    path::Path,
};

use oembed_type::{OembedData, OembedType};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use sha2::{Digest as _, Sha256};
use syn::{
    LitInt, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct OembedArgs {
    endpoint: LitStr,
    _comma_sep: Token![,],
    url: Punctuated<LitStr, Token![+]>,
    _comma_trailing: Option<Token![,]>,
}

impl Parse for OembedArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(OembedArgs {
            endpoint: input.parse()?,
            _comma_sep: input.parse()?,
            url: Punctuated::parse_terminated(input)?,
            _comma_trailing: input.parse()?,
        })
    }
}

#[derive(thiserror::Error, Debug)]
enum OembedError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Bad reply: {0}")]
    BadReplyStatus(reqwest::StatusCode),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

fn print_error(d: impl std::fmt::Display) -> TokenStream {
    let f = format!("Error: {d}");
    quote! { compile_error!(#f); }.into()
}

#[proc_macro]
pub fn oembed(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as OembedArgs);
    match args.fetch() {
        Ok(result) => result,
        Err(err) => print_error(err),
    }
}

struct RemoveOnFailure<'a>(Option<&'a Path>);

impl<'a> Drop for RemoveOnFailure<'a> {
    fn drop(&mut self) {
        if let Some(path) = self.0 {
            _ = remove_file(path);
        }
    }
}

impl OembedArgs {
    fn fetch(&self) -> Result<TokenStream, OembedError> {
        let endpoint = self.endpoint.value();
        let url: String = self.url.iter().map(|x| x.value()).collect();
        let mut endpoint_hash = Sha256::new();
        endpoint_hash.update(&endpoint);
        let mut url_hash = Sha256::new();
        url_hash.update(&url);
        let mut hash = Sha256::new();
        hash.update(endpoint_hash.finalize());
        hash.update(url_hash.finalize());
        let hash = {
            let bhash = hash.finalize();
            let mut output = "".to_string();
            for b in bhash.into_iter() {
                output.push_str(&format!("{:02x}", b));
            }
            output
        };

        let output_dir = Path::new("target/oembed");
        std::fs::create_dir_all(output_dir)?;
        let output_file = {
            let mut file = output_dir.to_path_buf();
            file.push(format!("{hash}.json"));
            file
        };

        let mut content = match OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&output_file)
        {
            Ok(mut target) => {
                target.lock()?;
                let mut remove_on_failure = RemoveOnFailure(Some(&output_file));
                let mut endpoint_url = reqwest::Url::parse(&endpoint)?;
                {
                    let mut query = endpoint_url.query_pairs_mut();
                    query.append_pair("url", &url);
                    query.append_pair("format", "json");
                }
                let mut rsp = reqwest::blocking::get(endpoint_url)?;
                let status = rsp.status();
                if !status.is_success() {
                    return Err(OembedError::BadReplyStatus(status));
                }

                rsp.copy_to(&mut target)?;
                remove_on_failure.0 = None;
                target.unlock()?;
                Ok(target)
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                let mut existing = OpenOptions::new()
                    .create(false)
                    .read(true)
                    .write(false)
                    .open(output_file)?;
                loop {
                    existing.lock_shared()?;
                    if existing.seek(SeekFrom::End(0))? > 0 {
                        break;
                    }
                    existing.unlock()?;
                }

                Ok(existing)
            }
            Err(e) => Err(e),
        }?;
        content.seek(SeekFrom::Start(0))?;
        let mut data = vec![];
        content.read_to_end(&mut data)?;
        let o: OembedData = serde_json::from_slice(&data[..])?;
        let os = [
            ("title", o.title),
            ("author_name", o.author_name),
            ("author_url", o.author_url),
            ("provider_name", o.provider_name),
            ("provider_url", o.provider_url),
            ("thumbnail_url", o.thumbnail_url),
        ];
        let data_unsigned = [
            ("cache_age", o.cache_age),
            ("thumbnail_width", o.thumbnail_width),
            ("thumbnail_height", o.thumbnail_height),
        ];
        let strings = os.into_iter().map(|(field, value)| {
            let field = Ident::new(field, Span::call_site());
            let value = value
                .map(|value| {
                    let value = LitStr::new(&value, Span::call_site());
                    quote! {
                        std::option::Option::Some(std::borrow::Cow::Borrowed( #value ))
                    }
                })
                .unwrap_or(quote! {
                    None
                });
            quote! {
                #field: #value
            }
        });
        let unsigneds = data_unsigned.into_iter().map(|(field, value)| {
            let field = Ident::new(field, Span::call_site());
            let value = value
                .map(|value| {
                    let value = LitInt::new(&value.to_string(), Span::call_site());
                    quote! {
                        std::option::Option::Some(#value )
                    }
                })
                .unwrap_or(quote! {
                    None
                });
            quote! {
                #field: #value
            }
        });
        let oembed_type = match o.content {
            OembedType::Video {
                html,
                width,
                height,
            } => {
                let (html, width, height) = (
                    LitStr::new(&html, Span::call_site()),
                    LitInt::new(&width.to_string(), Span::call_site()),
                    LitInt::new(&height.to_string(), Span::call_site()),
                );
                quote! {
                    OembedType::Video {
                        html: std::borrow::Cow::Borrowed( #html ),
                        width: #width,
                        height: #height,
                    }
                }
            }
        };
        Ok(quote! {
            const {
                use oembed::*;
                OembedData {
                    version: OembedVersion::Ver1_0,
                    #( #strings , )*
                    #( #unsigneds , )*
                    content: #oembed_type,
                }
            }
        }
        .into())
    }
}
