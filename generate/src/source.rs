//! Module implements reading of contract artifacts from various sources.

use crate::util;
use anyhow::{anyhow, Context, Error, Result};
use ethcontract_common::Address;
use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use url::Url;

/// A source of a Truffle artifact JSON.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Source {
    /// A Truffle artifact located on the local file system.
    Local(PathBuf),
    /// An address of a mainnet contract that has been verified on Etherscan.io.
    Etherscan(Address),
}

impl Source {
    /// Parses an artifact source from a string.
    ///
    /// Contract artifacts can be retrieved from the local filesystem or online
    /// from `etherscan.io`, this method parses artifact source URLs and accepts
    /// the following:
    /// - `relative/path/to/Contract.json`: a relative path to a truffle
    ///   artifact JSON file. This relative path is rooted in the current
    ///   working directory. To specify the root for relative paths, use
    ///   `Source::with_root`.
    /// - `/absolute/path/to/Contract.json` or
    ///   `file:///absolute/path/to/Contract.json`: an absolute path or file URL
    ///   to a truffle artifact JSON file.
    /// - `etherscan:0xXX..XX` or `https://etherscan.io/address/0xXX..XX`: a
    ///   address or URL of a verified contract on Etherscan.
    pub fn parse<S>(source: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let root = env::current_dir()?.canonicalize()?;
        Source::with_root(root, source)
    }

    /// Parses an artifact source from a string and a specified root directory
    /// for resolving relative paths. See `Source::with_root` for more details
    /// on supported source strings.
    pub fn with_root<P, S>(root: P, source: S) -> Result<Self>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let base = Url::from_directory_path(root)
            .map_err(|_| anyhow!("root path '{}' is not absolute"))?;
        let url = base.join(source.as_ref())?;

        match url.scheme() {
            "file" => Ok(Source::local(url.path())),
            "etherscan" => Source::etherscan(url.path()),
            "http" | "https" => match url.host_str() {
                Some("etherscan.io") => Source::etherscan(
                    url.path()
                        .rsplit('/')
                        .next()
                        .ok_or_else(|| anyhow!("HTTP URL does not have a path"))?,
                ),
                _ => Err(anyhow!("unsupported HTTP URL '{}'", url)),
            },
            _ => Err(anyhow!("unsupported URL '{}'", url)),
        }
    }

    /// Creates a local filesystem source from a path string.
    pub fn local<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Source::Local(path.as_ref().into())
    }

    /// Creates an Etherscan source from an address string.
    pub fn etherscan<S>(address: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let address =
            util::parse_address(address).context("failed to parse address for Etherscan source")?;
        Ok(Source::Etherscan(address))
    }

    /// Retrieves the source JSON of the artifact this will either read the JSON
    /// from the file system or retrieve a contract ABI from the network
    /// dependending on the source type.
    pub fn artifact_json(&self) -> Result<String> {
        match self {
            Source::Local(path) => read_json(path),
            Source::Etherscan(address) => get_etherscan_contract(*address),
        }
    }
}

impl FromStr for Source {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Source::parse(s)
    }
}

/// Reads a Truffle artifact JSON file from the local filesystem.
fn read_json(path: &Path) -> Result<String> {
    let path = if path.is_relative() {
        let absolute_path = path.canonicalize().with_context(|| {
            format!(
                "unable to canonicalize file from working dir {} with path {}",
                env::current_dir()
                    .map(|cwd| cwd.display().to_string())
                    .unwrap_or_else(|err| format!("??? ({})", err)),
                path.display(),
            )
        })?;
        Cow::Owned(absolute_path)
    } else {
        Cow::Borrowed(path)
    };

    let json = fs::read_to_string(path).context("failed to read artifact JSON file")?;
    Ok(json)
}

/// Retrieves a contract ABI from the Etherscan HTTP API and wraps it in an
/// artifact JSON for compatibility with the code generation facilities.
fn get_etherscan_contract(address: Address) -> Result<String> {
    // NOTE: We do not retrieve the bytecode since deploying contracts with the
    //   same bytecode is unreliable as the libraries have already linked and
    //   probably don't reference anything when deploying on other networks.

    let abi_url = format!(
        "http://api.etherscan.io/api?module=contract&action=getabi&address={:?}&format=raw",
        address,
    );
    let abi = util::http_get(&abi_url).context("failed to retrieve ABI from Etherscan.io")?;

    // NOTE: Wrap the retrieved ABI in an empty contract, this is because
    //   currently, the code generation infrastructure depends on having an
    //   `Artifact` instance.
    let json = format!(
        r#"{{"abi":{},"networks":{{"1":{{"address":"{:?}"}}}}}}"#,
        abi, address,
    );

    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_source() {
        let root = "/rooted";
        for (url, expected) in &[
            (
                "relative/Contract.json",
                Source::local("/rooted/relative/Contract.json"),
            ),
            (
                "/absolute/Contract.json",
                Source::local("/absolute/Contract.json"),
            ),
            (
                "etherscan:0x0001020304050607080910111213141516171819",
                Source::etherscan("0x0001020304050607080910111213141516171819").unwrap(),
            ),
            (
                "https://etherscan.io/address/0x0001020304050607080910111213141516171819",
                Source::etherscan("0x0001020304050607080910111213141516171819").unwrap(),
            ),
        ] {
            let source = Source::with_root(root, url).unwrap();
            assert_eq!(source, *expected);
        }
    }
}
