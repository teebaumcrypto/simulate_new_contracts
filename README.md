# Project leveraging anvil from foundry to simulate new token contract deployments

Goal:
1. fork chain at blocknumber
2. modify state, accounts etc
3. create uniswap LP pool (univ2)
4. initialize contract with predefined open Trading functions via owner
5. add liquidity / maybe just state modify?
6.Iterate through 10 blocks
6.1. execute swap for fixed amount of tokens -> get balance after -> Tax Buy
6.2. sell these tokens to the pool -> get balance after of pool -> Tax Sell


foundry link:
https://github.com/foundry-rs/foundry

I do this just for fun, don't expect it to work flawlessly