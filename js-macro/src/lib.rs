extern crate proc_macro;
use proc_macro::TokenStream;
use std::sync::Arc;

#[proc_macro]
pub fn minify_js(input: TokenStream) -> TokenStream {
    let js = format!("{}", input);
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
    proc_macro::TokenTree::Literal(proc_macro::Literal::string(
        &output.expect("Always minifies").code,
    ))
    .into()
}
