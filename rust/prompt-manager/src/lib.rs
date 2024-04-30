#![feature(proc_macro_quote)]

extern crate proc_macro;
use std::rc::Rc;

use parse_format::{ParseMode, Parser, Position};
use proc_macro::TokenStream;
use quote::quote;
use proc_macro2::{Ident, Span};
use syn::{parse::Parse, parse::ParseStream, ExprPath, LitStr, Token};
use unindent::unindent;

struct MacroInput {
    func_name: Ident,
    prompt: String,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let func_name: ExprPath = input.parse()?;
        // consume token: ,
        input.parse::<Token![,]>()?;
        let prompt: LitStr = input.parse()?;

        let segments = func_name.path.segments;
        assert!(segments.len() == 1, "Function name is required");

        Ok(Self {
            func_name: segments.first().unwrap().ident.clone(),
            prompt: unindent(prompt.value().trim()),
        })
    }
}

#[proc_macro]
pub fn make_prompt_template(input: TokenStream) -> TokenStream {
    let MacroInput { func_name, prompt } = syn::parse_macro_input!(input as MacroInput);

    let parser = Parser::new(&prompt, None, None, false, ParseMode::Format);
    // Pick out arguments only and emit literal string
    let args = parser
        .filter_map(|piece| {
        match piece{
            parse_format::Piece::String(_) => None,
            parse_format::Piece::NextArgument(arg) => Some(arg),
        }})
        .collect::<Rc<_>>();

    if args.iter().any(|arg| !matches!(arg.position, Position::ArgumentNamed(_))) {
        panic!("Only named arguments are supported, e.g. {{name}}")
    }

    // Convert args list to Ident list
    let args = args
        .iter()
        .map(|arg| {
            let Position::ArgumentNamed(name) = arg.position else {
                unreachable!("Only named arguments are supported, e.g. {{name}}");
            };

            Ident::new(name, Span::call_site())
        })
        .collect::<Vec<_>>();

    // Generate function define
    let expanded_macro = quote! {
		pub fn #func_name(#(#args: &str),*) -> String {
            // named arguments only
			::std::format!(#prompt, #(#args = #args),*)
		}
    };

    expanded_macro.into()
}

#[proc_macro]
pub fn make_func(ident: TokenStream) -> TokenStream {
    let func_name = format!("test_{}", ident.to_string());
    let ident_func_name = Ident::new(&func_name, Span::call_site());
    let exptened = quote! {
        pub fn #ident_func_name() {
            println!("call from {}", #func_name);
        }
    };

    exptened.into()
}
