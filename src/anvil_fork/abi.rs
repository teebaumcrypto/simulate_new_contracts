use ethers::prelude::abigen;

abigen!(
    UniswapV2Factory,
    r#"[
        createPair(address tokenA, address tokenB)(address)
        getPair(address tokenA, address tokenB)(address)
    ]"#,
);

abigen!(
    UniswapV2Router,
    r#"[
        function addLiquidity(address tokenA, address tokenB, uint amountADesired, uint amountBDesired, uint amountAMin, uint amountBMin, address to, uint deadline) external
        function addLiquidityETH(address token, uint amountTokenDesired, uint amountTokenMin, uint amountETHMin, address to, uint deadline) external
    ]"#,
);

abigen!(
    TokenContract,
    r#"[
        balanceOf(address)(uint256)
        owner()(address)
        approve(address spender, uint256 amount)(bool)
    ]"#,
);

abigen!(
    PairContract,
    r#"[
        function getReserves()(uint112, uint112, uint32)
    ]"#,
);
