//! Contracts API.

use std::ops::Not;

use frame_support::weights::Weight;
use frame_system::Config;
use pallet_contracts::{CollectEvents, DebugInfo, Determinism};
use pallet_contracts_primitives::{
    Code, CodeUploadResult, ContractExecResult, ContractInstantiateResult,
};
use parity_scale_codec::Decode as _;

use crate::{
    runtime::{AccountIdFor, Runtime},
    EventRecordOf, Sandbox,
};

/// Interface for contract-related operations.
pub trait ContractApi<R: Runtime> {
    /// Interface for `bare_instantiate` contract call with a simultaneous upload.
    ///
    /// # Arguments
    ///
    /// * `contract_bytes` - The contract code.
    /// * `value` - The number of tokens to be transferred to the contract.
    /// * `data` - The input data to be passed to the contract (including constructor name).
    /// * `salt` - The salt to be used for contract address derivation.
    /// * `origin` - The sender of the contract call.
    /// * `gas_limit` - The gas limit for the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    #[allow(clippy::too_many_arguments)]
    fn deploy_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        value: u128,
        data: Vec<u8>,
        salt: Vec<u8>,
        origin: AccountIdFor<R>,
        gas_limit: Weight,
        storage_deposit_limit: Option<u128>,
    ) -> ContractInstantiateResult<AccountIdFor<R>, u128, EventRecordOf<R>>;

    /// Interface for `bare_instantiate` contract call for a previously uploaded contract.
    ///
    /// # Arguments
    ///
    /// * `code_hash` - The code hash of the contract to instantiate.
    /// * `value` - The number of tokens to be transferred to the contract.
    /// * `data` - The input data to be passed to the contract (including constructor name).
    /// * `salt` - The salt to be used for contract address derivation.
    /// * `origin` - The sender of the contract call.
    /// * `gas_limit` - The gas limit for the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    #[allow(clippy::too_many_arguments)]
    fn instantiate_contract(
        &mut self,
        code_hash: Vec<u8>,
        value: u128,
        data: Vec<u8>,
        salt: Vec<u8>,
        origin: AccountIdFor<R>,
        gas_limit: Weight,
        storage_deposit_limit: Option<u128>,
    ) -> ContractInstantiateResult<AccountIdFor<R>, u128, EventRecordOf<R>>;

    /// Interface for `bare_upload_code` contract call.
    ///
    /// # Arguments
    ///
    /// * `contract_bytes` - The contract code.
    /// * `origin` - The sender of the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    fn upload_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        origin: AccountIdFor<R>,
        storage_deposit_limit: Option<u128>,
        determinism: Determinism,
    ) -> CodeUploadResult<<R as frame_system::Config>::Hash, u128>;

    /// Interface for `bare_call` contract call.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the contract to be called.
    /// * `value` - The number of tokens to be transferred to the contract.
    /// * `data` - The input data to be passed to the contract (including message name).
    /// * `origin` - The sender of the contract call.
    /// * `gas_limit` - The gas limit for the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    #[allow(clippy::too_many_arguments)]
    fn call_contract(
        &mut self,
        address: AccountIdFor<R>,
        value: u128,
        data: Vec<u8>,
        origin: AccountIdFor<R>,
        gas_limit: Weight,
        storage_deposit_limit: Option<u128>,
        determinism: Determinism,
    ) -> ContractExecResult<u128, EventRecordOf<R>>;
}

impl<R: Runtime> ContractApi<R> for Sandbox<R> {
    fn deploy_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        value: u128,
        data: Vec<u8>,
        salt: Vec<u8>,
        origin: AccountIdFor<R>,
        gas_limit: Weight,
        storage_deposit_limit: Option<u128>,
    ) -> ContractInstantiateResult<AccountIdFor<R>, u128, EventRecordOf<R>> {
        self.externalities.execute_with(|| {
            pallet_contracts::Pallet::<R>::bare_instantiate(
                origin,
                value,
                gas_limit,
                storage_deposit_limit,
                Code::Upload(contract_bytes),
                data,
                salt,
                DebugInfo::UnsafeDebug,
                CollectEvents::UnsafeCollect,
            )
        })
    }

    fn instantiate_contract(
        &mut self,
        code_hash: Vec<u8>,
        value: u128,
        data: Vec<u8>,
        salt: Vec<u8>,
        origin: AccountIdFor<R>,
        gas_limit: Weight,
        storage_deposit_limit: Option<u128>,
    ) -> ContractInstantiateResult<AccountIdFor<R>, u128, EventRecordOf<R>> {
        let mut code_hash = &code_hash[..];
        self.externalities.execute_with(|| {
            pallet_contracts::Pallet::<R>::bare_instantiate(
                origin,
                value,
                gas_limit,
                storage_deposit_limit,
                Code::Existing(
                    <R as Config>::Hash::decode(&mut code_hash).expect("Invalid code hash"),
                ),
                data,
                salt,
                DebugInfo::UnsafeDebug,
                CollectEvents::UnsafeCollect,
            )
        })
    }

    fn upload_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        origin: AccountIdFor<R>,
        storage_deposit_limit: Option<u128>,
        determinism: Determinism,
    ) -> CodeUploadResult<<R as frame_system::Config>::Hash, u128> {
        self.externalities.execute_with(|| {
            pallet_contracts::Pallet::<R>::bare_upload_code(
                origin,
                contract_bytes,
                storage_deposit_limit,
                determinism,
            )
        })
    }

    fn call_contract(
        &mut self,
        address: AccountIdFor<R>,
        value: u128,
        data: Vec<u8>,
        origin: AccountIdFor<R>,
        gas_limit: Weight,
        storage_deposit_limit: Option<u128>,
        determinism: Determinism,
    ) -> ContractExecResult<u128, EventRecordOf<R>> {
        self.externalities.execute_with(|| {
            pallet_contracts::Pallet::<R>::bare_call(
                origin,
                address,
                value,
                gas_limit,
                storage_deposit_limit,
                data,
                DebugInfo::UnsafeDebug,
                CollectEvents::UnsafeCollect,
                determinism,
            )
        })
    }
}

/// Converts bytes to a '\n'-split string, ignoring empty lines.
pub fn decode_debug_buffer(buffer: &[u8]) -> Vec<String> {
    let decoded = buffer.iter().map(|b| *b as char).collect::<String>();
    decoded
        .split('\n')
        .filter_map(|s| s.is_empty().not().then_some(s.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use frame_support::sp_runtime::traits::Hash;
    use pallet_contracts::Origin;

    use super::*;
    use crate::{chain_api::ChainApi, minimal::RuntimeEvent, MinimalRuntime, DEFAULT_GAS_LIMIT};

    fn compile_module(contract_name: &str) -> Vec<u8> {
        let path = [
            std::env::var("CARGO_MANIFEST_DIR")
                .as_deref()
                .unwrap_or("drink"),
            "/test-resources/",
            contract_name,
            ".wat",
        ]
        .concat();
        wat::parse_file(path).expect("Failed to parse wat file")
    }

    #[test]
    fn can_upload_code() {
        let mut sandbox = Sandbox::<MinimalRuntime>::new().unwrap();
        let wasm_binary = compile_module("dummy");
        let hash = <<MinimalRuntime as frame_system::Config>::Hashing>::hash(&wasm_binary);

        let result = sandbox.upload_contract(
            wasm_binary,
            MinimalRuntime::default_actor(),
            None,
            Determinism::Enforced,
        );

        assert!(result.is_ok());
        assert_eq!(hash, result.unwrap().code_hash);
    }

    #[test]
    fn can_deploy_contract() {
        let mut sandbox = Sandbox::<MinimalRuntime>::new().unwrap();
        let wasm_binary = compile_module("dummy");

        let events_before = sandbox.get_current_block_events();
        assert!(events_before.is_empty());

        let result = sandbox.deploy_contract(
            wasm_binary,
            0,
            vec![],
            vec![],
            MinimalRuntime::default_actor(),
            DEFAULT_GAS_LIMIT,
            None,
        );
        assert!(result.result.is_ok());
        assert!(!result.result.unwrap().result.did_revert());

        let events = result.events.expect("Drink should collect events");
        let event_count = events.len();
        let instantiation_event = events[event_count - 2].clone();
        assert!(matches!(
            instantiation_event.event,
            RuntimeEvent::Contracts(pallet_contracts::Event::<MinimalRuntime>::Instantiated { .. })
        ));
        let deposit_event = events[event_count - 1].clone();
        assert!(matches!(
            deposit_event.event,
            RuntimeEvent::Contracts(
                pallet_contracts::Event::<MinimalRuntime>::StorageDepositTransferredAndHeld { .. }
            )
        ));
    }

    #[test]
    fn can_call_contract() {
        let mut sandbox = Sandbox::<MinimalRuntime>::new().unwrap();
        let actor = MinimalRuntime::default_actor();
        let wasm_binary = compile_module("dummy");

        let result = sandbox.deploy_contract(
            wasm_binary,
            0,
            vec![],
            vec![],
            actor.clone(),
            DEFAULT_GAS_LIMIT,
            None,
        );

        let contract_address = result
            .result
            .expect("Contract should be deployed")
            .account_id;

        sandbox.reset_current_block_events();

        let result = sandbox.call_contract(
            contract_address.clone(),
            0,
            vec![],
            actor.clone(),
            DEFAULT_GAS_LIMIT,
            None,
            Determinism::Enforced,
        );
        assert!(result.result.is_ok());
        assert!(!result.result.unwrap().did_revert());

        let events = result.events.expect("Drink should collect events");
        assert!(events.len() == 2);

        assert_eq!(
            events[0].event,
            RuntimeEvent::Contracts(pallet_contracts::Event::<MinimalRuntime>::ContractEmitted {
                contract: contract_address.clone(),
                data: vec![0, 0, 0, 0],
            })
        );

        assert_eq!(
            events[1].event,
            RuntimeEvent::Contracts(pallet_contracts::Event::<MinimalRuntime>::Called {
                contract: contract_address,
                caller: Origin::Signed(actor),
            }),
        );
    }
}
