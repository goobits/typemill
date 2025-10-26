//! Procedural macros for simplifying language plugin creation in TypeMill.
//!
//! This crate provides the `define_language_plugin!` macro, which generates
//! some of the boilerplate required for a new language plugin.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

// Represents the parsed arguments for the `define_language_plugin!` macro.
struct PluginArgs {
    plugin_struct: Ident,
    name: LitStr,
    extensions: Expr,
    manifest_filename: LitStr,
    source_dir: LitStr,
    entry_point: LitStr,
    module_separator: LitStr,
    capabilities: Expr,
    lsp_config: Option<ExprTuple>,
}

impl Parse for PluginArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let entries = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

        let mut plugin_struct = None;
        let mut name = None;
        let mut extensions = None;
        let mut manifest_filename = None;
        let mut source_dir = None;
        let mut entry_point = None;
        let mut module_separator = None;
        let mut capabilities = None;
        let mut lsp_config = None;

        for expr in entries {
            if let Expr::Assign(assign) = expr {
                if let Expr::Path(path) = *assign.left {
                    let key = path.path.get_ident().ok_or_else(|| {
                        Error::new_spanned(&path, "Expected an identifier for key")
                    })?;
                    let value = *assign.right;

                    if key == "plugin_struct" {
                        if let Expr::Path(val_path) = value {
                            plugin_struct = Some(val_path.path.get_ident().unwrap().clone());
                        }
                    } else if key == "name" {
                        if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
                            name = Some(s);
                        }
                    } else if key == "extensions" {
                        extensions = Some(value);
                    } else if key == "manifest_filename" {
                         if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
                            manifest_filename = Some(s);
                        }
                    } else if key == "source_dir" {
                         if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
                            source_dir = Some(s);
                        }
                    } else if key == "entry_point" {
                         if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
                            entry_point = Some(s);
                        }
                    } else if key == "module_separator" {
                         if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
                            module_separator = Some(s);
                        }
                    } else if key == "capabilities" {
                        capabilities = Some(value);
                    } else if key == "lsp_config" {
                        if let Expr::Tuple(tuple) = value {
                            lsp_config = Some(tuple);
                        }
                    }
                }
            }
        }

        Ok(PluginArgs {
            plugin_struct: plugin_struct.ok_or_else(|| input.error("missing `plugin_struct` field"))?,
            name: name.ok_or_else(|| input.error("missing `name` field"))?,
            extensions: extensions.ok_or_else(|| input.error("missing `extensions` field"))?,
            manifest_filename: manifest_filename.ok_or_else(|| input.error("missing `manifest_filename` field"))?,
            source_dir: source_dir.ok_or_else(|| input.error("missing `source_dir` field"))?,
            entry_point: entry_point.ok_or_else(|| input.error("missing `entry_point` field"))?,
            module_separator: module_separator.ok_or_else(|| input.error("missing `module_separator` field"))?,
            capabilities: capabilities.ok_or_else(|| input.error("missing `capabilities` field"))?,
            lsp_config,
        })
    }
}

/// A procedural macro to generate boilerplate for a TypeMill language plugin.
///
/// This macro generates:
/// - `METADATA` and `CAPABILITIES` constants.
/// - A `new()` factory function.
/// - The `mill_plugin!` registration call.
#[proc_macro]
pub fn define_language_plugin(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as PluginArgs);

    let plugin_struct = &args.plugin_struct;
    let name = &args.name;
    let extensions = &args.extensions;
    let manifest_filename = &args.manifest_filename;
    let source_dir = &args.source_dir;
    let entry_point = &args.entry_point;
    let module_separator = &args.module_separator;
    let capabilities = &args.capabilities;

    let lsp_config_token = match &args.lsp_config {
        Some(config) => {
            let server_name = &config.elems[0];
            let server_args = &config.elems[1];
            quote! { Some(::mill_plugin_api::LspConfig::new(#server_name, #server_args)) }
        },
        None => quote! { None },
    };

    let generated_code = quote! {
        // --- Generated by define_language_plugin! macro ---

        impl #plugin_struct {
            /// Static metadata for the language.
            pub const METADATA: ::mill_plugin_api::LanguageMetadata = ::mill_plugin_api::LanguageMetadata {
                name: #name,
                extensions: &#extensions,
                manifest_filename: #manifest_filename,
                source_dir: #source_dir,
                entry_point: #entry_point,
                module_separator: #module_separator,
            };

            /// The capabilities of this plugin, defined by the user.
            pub const CAPABILITIES: ::mill_plugin_api::PluginCapabilities = #capabilities;

            /// Creates a new, boxed instance of the plugin.
            #[allow(clippy::new_ret_no_self)]
            pub fn new() -> Box<dyn ::mill_plugin_api::LanguagePlugin> {
                Box::new(Self::default())
            }
        }

        ::mill_plugin_api::mill_plugin! {
            name: #name,
            extensions: #extensions,
            manifest: #manifest_filename,
            capabilities: #plugin_struct::CAPABILITIES,
            factory: #plugin_struct::new,
            lsp: #lsp_config_token
        }
    };

    TokenStream::from(generated_code)
}