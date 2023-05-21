use std::{time::Duration};

use anvil::spawn;
use anyhow::Result;
use simulate_new_contracts::{anvil_fork::localfork::fork_config, preload_lazy_static};
use tokio::runtime::Runtime;
use tracing::info;

fn main() -> Result<()> {
    // preload all global variables 
    preload_lazy_static();
    
    // Create the runtime
    // we don't want to run full async
    let rt: Runtime = Runtime::new().unwrap();

    // spawn a blocked thread so the program won't quit
    let _ = rt.block_on(async move{
        let (api, handle) = spawn(fork_config()).await;
        
        let _provider = handle.http_provider();
        //let addr = H160::from_str("0x213414123123").unwrap();
        //let balance = api.balance(addr, None).await.unwrap();
        //let provider_balance = provider.get_balance(addr, None).await.unwrap();
        let block = api.block_number().unwrap();
        info!("block api:      {}", block);
        let _ = api.evm_mine(None).await;

        tokio::time::sleep(Duration::from_millis(1500)).await;
        let block = api.block_number().unwrap();
        info!("block api:      {}", block);

        
        info!("executed successfully");
    });


    info!("Finished everything");
    Ok(())
}
