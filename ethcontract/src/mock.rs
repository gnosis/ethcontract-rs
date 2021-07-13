//!

use crate::json::Value;
use crate::jsonrpc::Call;
use crate::web3::{Error, RequestId};
use web3::Transport;
use crate::{Web3, Instance, Address};
use crate::common::Abi;
use crate::contract::MethodDefaults;
use crate::dyns::{DynWeb3, DynTransport, DynInstance};
use crate::tokens::Tokenize;
use crate::common::hash::H32;

pub struct Mock;
impl Mock {
    pub fn web3(&self) -> &DynWeb3 { todo!() }
    pub fn transport(&self) -> &DynTransport { todo!() }
    pub fn deploy(&self, abi: Abi) -> MockContract { todo!() }
}

pub struct MockContract;
impl MockContract {
    pub fn web3(&self) -> &DynWeb3 { todo!() }
    pub fn transport(&self) -> &DynTransport { todo!() }
    pub fn abi(&self) -> &Abi { todo!() }
    pub fn address(&self) -> Address { todo!() }
    pub fn defaults(&self) -> &MethodDefaults { todo!() }
    pub fn defaults_mut(&mut self) -> &mut MethodDefaults { todo!() }
    pub fn instance(&self) -> &DynInstance { todo!() }
    pub fn instance_mut(&self) -> &mut DynInstance { todo!() }
    pub fn into_instance(self) -> DynInstance { todo!() }

    #[must_use]
    pub fn expect_method<A, R>(&self, signature: H32) -> MockMethod<A, R> { todo!() }
    #[must_use]
    pub fn expect_view_method<A, R>(&self, signature: H32) -> MockMethodView<A, R> { todo!() }
    #[must_use]
    pub fn expect_fallback(&self) -> MockMethod<(), ()> { todo!() }

    pub fn checkpoint(&mut self) { todo!() }
}

struct MockMethod<A: Tokenize, R: Tokenize>;
impl<A, R> MockMethod<A, R> {
    pub fn in_sequence(self, seq: mockall::Sequence) -> Self { todo!() }
    pub fn times(self, times: impl Into<mockall::TimesRange>) -> Self { todo!() }
    pub fn never(self) -> Self { todo!() }
    pub fn once(self) -> Self { todo!() }

    // verify inputs
    // verify tx settings
    // returns / reverts
    // confirmations (transport will have to implement `eth_blockNumber`)
}

struct MockMethodView<A: Tokenize, R: Tokenize>;
// TODO: confirmations!

#[derive(Clone, Debug)]
struct MockTransport;
impl Transport for MockTransport {
    type Out = futures::future::Ready<Result<Value, Error>>;
    fn prepare(&self, method: &str, params: Vec<Value>) -> (RequestId, Call) { todo!() }
    fn send(&self, id: RequestId, request: Call) -> Self::Out { todo!() }
}
