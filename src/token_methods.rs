use anyhow::{Result, anyhow};
use std::sync::Arc;
use ethers::{types::{H160, U256}};

use crate::anvil_fork::abi::TokenContract;

/// get the owner balance:
/// check if creator is owner -> return(owner,balanceOf(owner))
/// if creator isn't owner -> check if balance of owner > zero -> return(owner,balanceOf(owner))
/// if balance of owner == zero && balance of creator != zero -> return(creator,balanceOf(creator))
/// if balance of creator & owner == zero -> return (zero, zero)
pub async fn get_owner_with_balance(provider: Arc<ethers::providers::Provider<ethers::providers::Http>>, token: H160, creator: H160) -> Result<(H160, U256)> {
    // initiate a token contract with a default ABI
    let token = TokenContract::new(token, Arc::clone(&provider));
    // check if creator is owner of contract
    if let Ok(owner) = token.owner().call().await {
        // if the owner is not the creator, we need to find the acc with balance of the token
        if owner != creator {
            // check if owner or creator has balance
            if let Ok(balance_owner) = token.balance_of(owner).call().await {
                // if the balance of the owner is bigger than zero, we take this
                if balance_owner > U256::zero() {
                    return Ok((owner, balance_owner))
                }
                // if balance owner isn't bigger than 0 we check balance of creator
                else if let Ok(balance_creator) = token.balance_of(creator).call().await {
                    if balance_creator > U256::zero() {
                        return Ok((owner, balance_creator))
                    } else {
                        return Err(anyhow!("Balance of creator and owner is zero"))
                    }
                } else {
                    return Err(anyhow!("Couldn't get balance of creator and owner balance is zero"))
                }
            } 
            // we couldn't get balance of owner via call, which should indicate it's not a correct erc20
            else {
                return Err(anyhow!("not correct erc20"))
            }
        }
        // creator is owner
        else {
            // get balance of owner
            if let Ok(balance_owner) = token.balance_of(owner).call().await {
                if balance_owner > U256::zero() {
                    return Ok((owner, balance_owner))
                } else {
                    return Err(anyhow!("Balance of owner is zero"))
                }
            } else {
                return Err(anyhow!("not correct erc20"))
            }
        }
    } else {
        // we didn't receive an owner because function did not exist
        // get balance of owner
        if let Ok(balance_creator) = token.balance_of(creator).call().await {
            if balance_creator > U256::zero() {
                return Ok((creator, balance_creator))
            } else {
                return Err(anyhow!("Balance of owner is zero"))
            }
        } else {
            return Err(anyhow!("balance of function does not exist"))
        }
    }

}