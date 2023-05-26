use std::{str::FromStr, sync::Arc, ops::{Mul, Div}};

use anvil::spawn;
use anyhow::Result;
use ethers::{
    providers::Middleware,
    types::{transaction::eip2718::TypedTransaction, H160, U256, Address},
};
use simulate_new_contracts::{
    anvil_fork::{
        abi::{TokenContract, UniswapV2Router, PairContract, UniswapV2Factory},
        localfork::fork_config,
    },
    preload_lazy_static,
    token_methods::{create_and_send_tx, get_owner_with_balance},
    SETTINGS, ONE_ETH, TEN_ETH,
};
use tokio::runtime::Runtime;
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::from_default_env()
            .add_directive("node=off".parse().unwrap())
        )
        .init();

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
        let real_owner: H160;
        let balance: U256;
        // create a anvil fork on block number
        let (api, handle) = spawn(fork_config(block_number)).await;

        // get the http provider handle for making calls
        let provider: Arc<ethers::providers::Provider<ethers::providers::Http>> =
            Arc::new(handle.http_provider());
        
        // get the real owner with balance
        if let Ok(tuple) = get_owner_with_balance(provider.clone(), token, creator).await {
            real_owner = tuple.0;
            balance = tuple.1;
        } else {
            panic!("Couldn't fetch owner with balance");
        }

        // impersonate real owner
        api.anvil_impersonate_account(real_owner).await.unwrap();

        // add 10 eth for real owner
        api.anvil_set_balance(real_owner, *TEN_ETH)
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
        match create_and_send_tx(Arc::clone(&provider), tx, real_owner, None).await {
            Ok(_) => info!("approval tx ok, waiting for new block"),
            Err(e) => warn!("failed with error: {:?}", e)
        }


        // add Liquidity: impersonate real owner and add liquidity
        // create router with abi to interact (addLiquidity)
        let uniswap_router = UniswapV2Router::new(SETTINGS.router, Arc::clone(&provider));
        let add_liquidity_call = uniswap_router.add_liquidity_eth(
            token,
            balance,
            balance,
            *ONE_ETH,
            real_owner,
            U256::from(1984669967u64),
        );
        // convert call to typed transaction
        let tx: TypedTransaction = add_liquidity_call.tx;
        // fill tx with infos + send it
        match create_and_send_tx(Arc::clone(&provider), tx, real_owner, Some(*ONE_ETH)).await {
            Ok(_) => info!("liquidity add tx ok, waiting for new block"),
            Err(e) => warn!("failed with error: {:?}", e)
        }

        // mine new block
        let _ = api.evm_mine(None).await;

        // initiate the uniswap factory contract with a factory ABI
        let factory = UniswapV2Factory::new(SETTINGS.factory, Arc::clone(&provider));
        // get pair address from the factory, WETH - TOKEN
        if let Ok(pair) = factory.get_pair(SETTINGS.weth, token).call().await {
            info!("pair: {:?}", pair);
            // initiate the uniswap pair contract with a pair ABI
            let pair = PairContract::new(pair, Arc::clone(&provider));
            if let Ok(reserves) = pair.get_reserves().call().await {
                info!("reserves: {:?}", reserves);
            }
        }
        
        // create approve transaction
        let set_trading_call = token_contract.set_trading(true);
        // convert call to typed transaction
        let tx: TypedTransaction = set_trading_call.tx;
        // fill tx with infos + send it
        let _ = create_and_send_tx(Arc::clone(&provider), tx, real_owner, None).await;


        // now we can check if we can execute swaps with another wallet
        let random_addr = Address::random();
        // impersonate the random address
        api.anvil_impersonate_account(random_addr).await.unwrap();
        // adding 10 eth for swapping
        api.anvil_set_balance(random_addr, *TEN_ETH)
            .await
            .unwrap();
        let swap_call = uniswap_router.swap_exact_eth_for_tokens(
            U256::from(1),
            vec![SETTINGS.weth,token],
            random_addr,
            U256::from(1984669967u64),
        );
        // convert call to typed transaction
        let swap_tx: TypedTransaction = swap_call.tx;
        match create_and_send_tx(Arc::clone(&provider), swap_tx, random_addr, Some(*ONE_ETH)).await {
            Ok(_) => info!("tx ok, waiting for new block"),
            Err(e) => warn!("failed with error: {:?}", e)
        }

        for i in (1..500).rev() {
            let swap_call = uniswap_router.swap_eth_for_exact_tokens(
                    U256::from(balance.mul(U256::from(i)).div(10000u32)),
                vec![SETTINGS.weth,token],
                random_addr,
                U256::from(1984669967u64),
            );
            // convert call to typed transaction
            let swap_tx: TypedTransaction = swap_call.tx;

            match create_and_send_tx(Arc::clone(&provider), swap_tx, random_addr, Some(*ONE_ETH)).await {
                Ok(_) => {
                    // mine new block
                    let _ = api.evm_mine(None).await;
                    
                    if let Ok(token_balance_random_addr) = token_contract.balance_of(random_addr).call().await {
                        info!("token-balance of random addr: {}", token_balance_random_addr);
                    }
                        info!("swap tx {i} ok, waiting for new block");
                        break;
                },
                    Err(_) => ()
            }
        }

        // mine new block
        let _ = api.evm_mine(None).await;

        info!("executed successfully");
    });

    info!("Finished everything");
    Ok(())
}
