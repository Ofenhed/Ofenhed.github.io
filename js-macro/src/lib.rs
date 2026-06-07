extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::{
    iter::{Extend as _, once},
    sync::Arc,
};

fn err_tokens(err: impl std::fmt::Display) -> TokenStream {
    let mut tokens = TokenStream::default();
    tokens.extend(once(Ident::new("compile_error", Span::call_site())));
    tokens.extend(once(Punct::new('!', Spacing::Joint)));
    let mut inner = TokenStream::default();
    inner.extend(once(Literal::string(&err.to_string())));
    tokens.extend(once(Group::new(Delimiter::Parenthesis, inner)));
    tokens
}

#[proc_macro]
pub fn minify_js(input: TokenStream) -> TokenStream {
    let js: String = input
        .into_iter()
        .map(|tree| {
            tree.span()
                .source_text()
                .unwrap_or_else(move || tree.to_string())
        })
        .collect();
    let cm = Arc::<swc_common::SourceMap>::default();

    let c = swc::Compiler::new(cm.clone());
    let output = swc_common::GLOBALS.set(&Default::default(), || {
        swc::try_with_handler(cm.clone(), Default::default(), |handler| {
            let fm = cm.new_source_file(Arc::new(swc_common::FileName::Anon), js.to_string());

            c.minify(
                fm,
                handler,
                &swc::config::JsMinifyOptions {
                    compress: swc::BoolOrDataConfig::from_bool(true),
                    mangle: swc::BoolOrDataConfig::from_bool(true),
                    module: swc::config::IsModule::Bool(false),
                    ..Default::default()
                },
                swc::JsMinifyExtras::default(),
            )
        })
    });
    match output {
        Ok(output) => TokenTree::Literal(Literal::string(&output.code)).into(),
        Err(err) => err_tokens(err.to_pretty_string()),
    }
}
