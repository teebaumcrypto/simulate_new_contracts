use ethers::prelude::abigen;

abigen!(
    UniswapV2Factory,
    r#"[
        function createPair(address tokenA, address tokenB) external
    ]"#,
);

abigen!(
    UniswapV2Router,
    r#"[
        function addLiquidity(address tokenA, address tokenB, uint amountADesired, uint amountBDesired, uint amountAMin, uint amountBMin, address to, uint deadline) external
        function addLiquidityETH(address token, uint amountTokenDesired, uint amountTokenMin, uint amountETHMin, address to, uint deadline) external
    ]"#,
);
