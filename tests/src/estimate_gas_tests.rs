use crate::common::{
    deploy_entry_point, deploy_simple_account_factory,
    gen::{EntryPointContract, SimpleAccountFactory},
    setup_database_mempool_reputation, setup_geth, setup_memory_mempool_reputation, ClientType,
    DeployedContract, SEED_PHRASE,
};
use alloy_chains::Chain;
use ethers::{
    providers::Middleware,
    signers::{coins_bip39::English, MnemonicBuilder, Signer},
    types::{transaction::eip2718::TypedTransaction, Bytes, H160, U256},
    utils::GethInstance,
};
use silius_contracts::EntryPoint;
use silius_mempool::{validate::validator::new_canonical, UoPool};
use silius_primitives::{UserOperation, Wallet as UoWallet};
use std::sync::Arc;

async fn setup_basic() -> eyre::Result<(
    Arc<ClientType>,
    DeployedContract<EntryPointContract<ClientType>>,
    u64,
    GethInstance,
    DeployedContract<SimpleAccountFactory<ClientType>>,
)> {
    let chain_id = 1337u64;
    let (geth, _client, _) = setup_geth().await?;
    let client = Arc::new(_client);
    let ep = deploy_entry_point(client.clone()).await?;
    let simple_account_factory = deploy_simple_account_factory(client.clone(), ep.address).await?;

    Ok((client.clone(), ep, chain_id, geth, simple_account_factory))
}

macro_rules! estimate_gas_with_init_code {
    ($setup:expr, $name: ident) => {
        #[tokio::test]
        async fn $name() -> eyre::Result<()> {
            let (client, entry_point, chain_id, _geth, simple_account_factory) =
                setup_basic().await?;
            let (mempool, reputation) = $setup;
            let max_verification_gas = 5000000.into();
            let chain = Chain::from_id(chain_id);
            let entry = EntryPoint::new(client.clone(), entry_point.address);
            let entry_for_uopool = EntryPoint::new(client.clone(), entry_point.address);
            let min_priority_fee_per_gas = 0.into();
            let validator =
                new_canonical(entry, chain, max_verification_gas, min_priority_fee_per_gas);
            let uopool = UoPool::new(
                entry_for_uopool,
                validator,
                mempool,
                reputation,
                max_verification_gas,
                chain,
                None,
            );

            let wallet = MnemonicBuilder::<English>::default().phrase(SEED_PHRASE).build()?;
            let owner_address = wallet.address();
            let address: H160 = simple_account_factory
                .contract()
                .get_address(owner_address, U256::from(1))
                .call()
                .await?;
            let nonce = client.get_transaction_count(owner_address, None).await?;
            let mut initial_fund = TypedTransaction::default();
            initial_fund
                .set_from(owner_address)
                .set_to(address)
                .set_value(1000000000000000u64)
                .set_nonce(nonce);
            let _receipt = client.send_transaction(initial_fund, None).await?.await?;
            let _balance = client.get_balance(address, None).await?;

            let call =
                simple_account_factory.contract().create_account(owner_address, U256::from(1));
            let tx: TypedTransaction = call.tx;
            let mut init_code = Vec::new();
            init_code.extend_from_slice(simple_account_factory.address.as_bytes());
            init_code.extend_from_slice(tx.data().unwrap().to_vec().as_slice());

            // This is the `execute(address dest, uint256 value, bytes calldata func)` call data
            // with all empty values.
            let call_data: Vec<u8> = vec![
                182, 29, 39, 246, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];

            let (_gas_price, priority_fee) = client.estimate_eip1559_fees(None).await?;
            let nonce = client.get_transaction_count(address, None).await?;
            let user_op = UserOperation {
                sender: address,
                nonce,
                init_code: Bytes::from(init_code),
                call_data: Bytes::from(call_data),
                call_gas_limit: U256::from(1),
                verification_gas_limit: U256::from(1000000u64),
                pre_verification_gas: U256::from(1),
                max_fee_per_gas: U256::from(1),
                max_priority_fee_per_gas: priority_fee,
                paymaster_and_data: Bytes::new(),
                signature: Bytes::default(),
            };

            let uo_wallet = UoWallet::from_phrase(SEED_PHRASE, &chain_id.into(), false)?;
            let user_op =
                uo_wallet.sign_uo(&user_op, &entry_point.address, &chain_id.into()).await?;

            let _ = uopool.estimate_user_operation_gas(&user_op).await.expect("estimate done");
            Ok(())
        }
    };
}

estimate_gas_with_init_code!(setup_database_mempool_reputation(), estimate_gas_init_code_datbase);
estimate_gas_with_init_code!(setup_memory_mempool_reputation(), estimate_gas_init_code_memory);
