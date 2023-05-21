use std::{str::FromStr, sync::Arc};

use anvil::spawn;
use anyhow::Result;
use ethers::{
    providers::Middleware,
    types::{transaction::eip2718::TypedTransaction, H160, U256},
};
use simulate_new_contracts::{
    anvil_fork::{
        abi::{TokenContract, UniswapV2Router},
        localfork::fork_config,
    },
    preload_lazy_static,
    token_methods::{create_and_send_tx, get_owner_with_balance},
    SETTINGS,
};
use tokio::runtime::Runtime;
use tracing::info;

fn main() -> Result<()> {
    // preload all global variables
    preload_lazy_static();

    /*
    Test token to fork and simulate stuff
    https://fomoape.com/#/analyze?token=0x2706fd8a70affe732e6c6955d8c47f875b754d2f
        0x8f70ccf7-setTrading(bool)	maxtx:	15000000 (1.50%)	    tax buy: 25%
                                    max_wallet:	15000000 (1.50%)	tax sell: 25%
    */
    // TODO: creator, token, block_number has to be from cli args
    let creator = H160::from_str("0x5b19282ee1a76b1caca1fc4c437a2499229459df").unwrap();
    let token = H160::from_str("0x2706fd8a70affe732e6c6955d8c47f875b754d2f").unwrap();
    let block_number = 17307367u64;

    // Create the runtime
    // we don't want to run full async
    let rt: Runtime = Runtime::new().unwrap();

    // spawn a blocked thread so the program won't quit
    rt.block_on(async move {
        let (api, handle) = spawn(fork_config(block_number)).await;

        let provider: Arc<ethers::providers::Provider<ethers::providers::Http>> =
            Arc::new(handle.http_provider());
        let real_owner: H160;
        let balance: U256;
        if let Ok(tuple) = get_owner_with_balance(provider.clone(), token, creator).await {
            real_owner = tuple.0;
            balance = tuple.1;
        } else {
            panic!("Couldn't fetch owner with balance");
        }
        println!("real_owner:    {:?}", real_owner);
        println!("balance owner: {:?}", balance);
        // impersonate real owner
        api.anvil_impersonate_account(real_owner).await.unwrap();

        // add 10 eth for creator address
        api.anvil_set_balance(real_owner, U256::from(10e18 as u64))
            .await
            .unwrap();
        let faked_balance = provider.get_balance(real_owner, None).await.unwrap();
        info!(
            "faked_balance: {} on block: {}",
            faked_balance,
            api.block_number().unwrap()
        );

        // create token contract instance via ABI
        let token_contract = TokenContract::new(token, Arc::clone(&provider));
        // create approve transaction
        let approve_call = token_contract.approve(SETTINGS.router, U256::MAX);
        // convert call to typed transaction
        let tx: TypedTransaction = approve_call.tx;
        // fill tx with infos + send it
        let _ = create_and_send_tx(Arc::clone(&provider), tx.clone(), real_owner, None).await;

        // add Liquidity: impersonate real owner and add liquidity
        // create router with abi to interact (addLiquidity)
        let uniswap_router = UniswapV2Router::new(SETTINGS.router, Arc::clone(&provider));
        let one_eth = U256::from(1000000000000000000u64);
        let add_liquidity_call = uniswap_router.add_liquidity_eth(
            token,
            balance,
            balance,
            one_eth,
            real_owner,
            U256::from(1984669967u64),
        );
        // convert call to typed transaction
        let tx: TypedTransaction = add_liquidity_call.tx;
        // fill tx with infos + send it
        let _ = create_and_send_tx(Arc::clone(&provider), tx, real_owner, Some(one_eth)).await;

        // mine new block
        let _ = api.evm_mine(None).await;
        info!("executed successfully");
    });

    info!("Finished everything");
    Ok(())
}
