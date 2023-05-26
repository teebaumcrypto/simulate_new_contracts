use std::{
    ops::{Div, Mul, Sub},
    str::FromStr,
    sync::Arc,
};

use anvil::spawn;
use anyhow::{anyhow, Result};
use ethers::{
    providers::Middleware,
    types::{
        transaction::eip2718::TypedTransaction, Address, Eip1559TransactionRequest, H160, U256,
    },
    utils::hex,
};
use simulate_new_contracts::{
    anvil_fork::{
        abi::{PairContract, TokenContract, UniswapV2Factory, UniswapV2Router},
        localfork::fork_config,
    },
    preload_lazy_static,
    token_methods::{create_and_send_tx, get_owner_with_balance},
    ONE_ETH, SETTINGS, TEN_ETH,
};
use tokio::runtime::Runtime;
use tracing::info;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

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
        let _ = simulate_contract(creator, token, block_number).await;
    });

    Ok(())
}

pub async fn simulate_contract(creator: H160, token: H160, block_number: u64) -> Result<()> {
        let real_owner: H160;
        let balance: U256;
        // create a anvil fork on block number
        let (api, handle) = spawn(fork_config(block_number)).await;

        // get the http provider handle for making calls
        let provider: Arc<ethers::providers::Provider<ethers::providers::Http>> =
            Arc::new(handle.http_provider());
        
    // create token contract instance via ABI
    let token_contract = TokenContract::new(token, Arc::clone(&provider));
    // get token infos
    let token_total_supply = token_contract.total_supply().call().await?;
    let token_decimals = token_contract.decimals().call().await?;
    let token_decimals_powed = U256::from(10u32).pow(token_decimals);

        // get the real owner with balance
        if let Ok(tuple) = get_owner_with_balance(provider.clone(), token, creator).await {
            real_owner = tuple.0;
            balance = tuple.1;
        } else {
            return Err(anyhow!("Couldn't fetch owner with balance"));
        }

        // impersonate real owner
        api.anvil_impersonate_account(real_owner).await?;

        // add 10 eth for real owner
        api.anvil_set_balance(real_owner, *TEN_ETH).await?;

        let faked_balance = provider.get_balance(real_owner, None).await?;
        info!(
            "faked_balance: {} on block: {}",
            faked_balance,
            api.block_number()?
        );

        // create approve transaction
        let approve_call = token_contract.approve(SETTINGS.router, U256::MAX);
        // convert call to typed transaction
        let tx: TypedTransaction = approve_call.tx;
        // fill tx with infos + send it
        match create_and_send_tx(Arc::clone(&provider), tx, real_owner, None).await {
            Ok(_) => info!("approval tx ok"),
            Err(e) => return Err(anyhow!("Approve failed with error: {e:?}")),
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
            Err(e) => return Err(anyhow!("Add Liquidity failed with error: {e:?}")),
        }

    // will be set via user input, direct calldata so it can be changed in runtime and doesn't have to be in the ABI
    let trading_open_hex_data = "0x8f70ccf70000000000000000000000000000000000000000000000000000000000000001";
    // convert into bytes with ethers hex crate, remove 0x
    let bytes = hex::decode(&trading_open_hex_data[2..])?;

    // create EIP1559 - TypedTransaction to send
    let tx_trading_open = TypedTransaction::Eip1559(Eip1559TransactionRequest {
        to: Some(ethers::types::NameOrAddress::Address(token)),
        data: Some(bytes.into()),
        ..Default::default()
    });

    // fill tx with infos + send it
    let _ = create_and_send_tx(Arc::clone(&provider), tx_trading_open, real_owner, None).await;

        // now we can check if we can execute swaps with another wallet
        let random_addr = Address::random();
        // impersonate the random address
        api.anvil_impersonate_account(random_addr).await?;
        // adding 10 eth for swapping
        api.anvil_set_balance(random_addr, *TEN_ETH).await?;

        let mut current_basispoint_amount: U256 = U256::zero();
        let mut current_basispoint = 0u32;

        for i in (1..500).rev() {
            let amount_out: U256;
            // if total supply is smaller than pow of decimals the total supply is < 1 (e.g. 0.001)
            if token_total_supply < token_decimals_powed {
                amount_out = U256::from(token_total_supply.mul(U256::from(i)).div(10000u32));
            } else {
                amount_out = (U256::from(token_total_supply.mul(U256::from(i)).div(10000u32)))
                    .sub(token_decimals_powed);
            }

            let swap_call = uniswap_router.swap_eth_for_exact_tokens(
                amount_out,
                vec![SETTINGS.weth, token],
                random_addr,
                U256::from(1984669967u64),
            );
            // convert call to typed transaction
            let swap_tx: TypedTransaction = swap_call.tx;

        match create_and_send_tx(Arc::clone(&provider), swap_tx, random_addr, Some(*ONE_ETH)).await
            {
                Ok(_) => {
                    current_basispoint_amount = amount_out;
                    current_basispoint = i;
                    break;
                }
                Err(_) => (),
            }
        }

        info!("current_basispoint: {}", current_basispoint);
        info!("current_basispoint_amount: {}", current_basispoint_amount);
        
        if current_basispoint == 0u32 {
            return Err(anyhow!("Swap couldn't get executed"));
        }

        // mine new block
        let _ = api.evm_mine(None).await;
        info!("mined new block");
                    
        if let Ok(token_balance_random_addr) = token_contract.balance_of(random_addr).call().await {
            info!(
                "token-balance of random addr: {}",
                token_balance_random_addr
            );
            // multiply amount real out with 100, divide by amount given in input
            // 100 - sub this, will give the percentage of fees
            // if output = input (example: 200 tokens= 20'000 / 200 = 100, 100-100 = 0 => 0% fee )
            let buy_fee = U256::from(100u32)
                .sub(token_balance_random_addr.mul(100u32) / current_basispoint_amount);
            info!("buy fee: {}", buy_fee);
        }

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

        info!("executed successfully");
    Ok(())
}
