# Project leveraging anvil from foundry to simulate new token contract deployments

Goal:
1. fork chain at blocknumber [done]
2. modify state, accounts etc [done]
3. create uniswap LP pool (univ2) [done]
4. initialize contract with predefined open Trading functions via owner [done]
5. add liquidity [done]

Todo:
Refactor code -> either:
1. make an endpoint to execute while running
2. make it that it only works on one token per start (move forking to lazy_static)

Get total supply of token and use it for max tx amount

Current questions:
1. how to get max tx amount, iterate 90% 50% 30% 20% 10% 9%... 5% 4.5% ... 3% 2.9%..
--> is this fast enough?

6.Iterate through 10 blocks
6.1. execute swap for fixed amount of tokens -> get balance after -> Tax Buy
6.2. sell these tokens to the pool -> get balance after of pool -> Tax Sell


foundry link:
https://github.com/foundry-rs/foundry

I do this just for fun, don't expect it to work flawlessly