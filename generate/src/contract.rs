#![deny(missing_docs)]

//! Crate for generating type-safe bindings to Ethereum smart contracts. This
//! crate is intended to be used either indirectly with the `ethcontract`
//! crate's `contract` procedural macro or directly from a build script.

mod common;
mod deployment;
mod methods;
mod types;

use crate::util;
use crate::Args;
use anyhow::{anyhow, Context as _, Result};
use ethcontract_common::{Address, Artifact};
use inflector::Inflector;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::Visibility;

/// Internal shared context for generating smart contract bindings.
pub(crate) struct Context {
    /// The artifact JSON as string literal.
    artifact_json: Literal,
    /// The parsed artifact.
    artifact: Artifact,
    /// The identifier for the runtime crate. Usually this is `ethcontract` but
    /// it can be different if the crate was renamed in the Cargo manifest for
    /// example.
    runtime_crate: Ident,
    /// The visibility for the generated module and re-exported contract type.
    visibility: Visibility,
    /// The name of the module as an identifier in which to place the contract
    /// implementation. Note that the main contract type gets re-exported in the
    /// root.
    contract_mod: Ident,
    /// The contract name as an identifier.
    contract_name: Ident,
    /// Additional contract deployments.
    deployments: HashMap<u32, Address>,
    /// Manually specified method aliases.
    method_aliases: HashMap<String, Ident>,
}

impl Context {
    /// Create a context from the code generation arguments.
    fn from_args(args: Args) -> Result<Self> {
        let (artifact_json, artifact) = {
            let artifact_json = args
                .artifact_source
                .artifact_json()
                .context("failed to get artifact JSON")?;

            let artifact = Artifact::from_json(&artifact_json)
                .with_context(|| format!("invalid artifact JSON '{}'", artifact_json))
                .with_context(|| {
                    format!(
                        "failed to parse artifact from source {:?}",
                        args.artifact_source,
                    )
                })?;

            (Literal::string(&artifact_json), artifact)
        };

        let raw_contract_name = if let Some(name) = args.contract_name_override.as_ref() {
            name
        } else if !artifact.contract_name.is_empty() {
            &artifact.contract_name
        } else {
            return Err(anyhow!(
                "contract artifact is missing a name, this can happen when \
                 using a source that does not provide a contract name such  as \
                 Etherscan; in this case the contract must be manually \
                 specified"
            ));
        };

        let runtime_crate = util::ident(&args.runtime_crate_name);
        let visibility = match args.visibility_modifier.as_ref() {
            Some(vis) => syn::parse_str(vis)?,
            None => Visibility::Inherited,
        };
        let contract_mod = if let Some(name) = args.contract_mod_override.as_ref() {
            util::ident(name)
        } else {
            util::ident(&raw_contract_name.to_snake_case())
        };
        let contract_name = util::ident(raw_contract_name);

        let mut method_aliases = HashMap::new();
        for (signature, alias) in args.method_aliases.into_iter() {
            let alias = syn::parse_str(&alias)?;
            if method_aliases.insert(signature.clone(), alias).is_some() {
                return Err(anyhow!(
                    "duplicate method signature '{}' in method aliases",
                    signature,
                ));
            }
        }

        Ok(Context {
            artifact_json,
            artifact,
            runtime_crate,
            visibility,
            contract_mod,
            contract_name,
            deployments: args.deployments,
            method_aliases,
        })
    }

    fn methods_struct_name(&self) -> Result<Ident> {
        Ok(syn::parse_str(&format!("{}Methods", self.contract_name,))?)
    }
}

#[cfg(test)]
impl Default for Context {
    fn default() -> Self {
        Context {
            artifact_json: Literal::string("{}"),
            artifact: Artifact::empty(),
            runtime_crate: util::ident("ethcontract"),
            visibility: Visibility::Inherited,
            contract_mod: util::ident("contract"),
            contract_name: util::ident("Contract"),
            deployments: HashMap::new(),
            method_aliases: HashMap::new(),
        }
    }
}

pub(crate) fn expand(args: Args) -> Result<TokenStream> {
    let cx = Context::from_args(args)?;
    let contract = expand_contract(&cx).context("error expanding contract from source")?;

    Ok(contract)
}

fn expand_contract(cx: &Context) -> Result<TokenStream> {
    let vis = &cx.visibility;
    let contract_mod = &cx.contract_mod;
    let contract_name = &cx.contract_name;

    let common = common::expand(cx)?;
    let deployment = deployment::expand(cx)?;
    let methods = methods::expand(cx)?;

    Ok(quote! {
        #[allow(dead_code)]
        #vis mod #contract_mod {
            #common
            #deployment
            #methods
        }
        #vis use self::#contract_mod::#contract_name;
    })
}
