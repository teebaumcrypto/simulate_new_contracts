use std::{str::FromStr, sync::Arc};

use anvil::spawn;
use anyhow::Result;
use ethers::{types::{H160, U256}, providers::Middleware};
use simulate_new_contracts::{anvil_fork::{localfork::fork_config}, preload_lazy_static};
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
    let _token = H160::from_str("0x2706fd8a70affe732e6c6955d8c47f875b754d2f").unwrap();
    let block_number = 17307367u64;    
    
    // Create the runtime
    // we don't want to run full async
    let rt: Runtime = Runtime::new().unwrap();

    // spawn a blocked thread so the program won't quit
    let _ = rt.block_on(async move{
        let (api, handle) = spawn(fork_config(block_number)).await;
        
        let provider = Arc::new(handle.http_provider());
        let block = api.block_number().unwrap();
        
        // TODO: check if creator is owner of contract, else take owner
        // impersonate real owner
        api.anvil_impersonate_account(creator).await.unwrap();
        
        let initial_balance = provider.get_balance(creator, None).await.unwrap();
        info!("initial balance: {} on block: {}", initial_balance, block);

        // mine new block
        let _ = api.evm_mine(None).await;

        // add 10 eth for creator address
        api.anvil_set_balance(creator, U256::from(10e18 as u64)).await.unwrap();
        let faked_balance = provider.get_balance(creator, None).await.unwrap();
        let block = api.block_number().unwrap();
        info!("faked_balance: {} on block: {}", faked_balance, block);

        info!("executed successfully");
    });


    info!("Finished everything");
    Ok(())
}
