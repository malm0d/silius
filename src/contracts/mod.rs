use ethers::prelude::*;

abigen!(
    AggregatedWallet,
    "$OUT_DIR/IAggregatedAccount.sol/IAggregatedAccount.json"
);

abigen!(Aggregator, "$OUT_DIR/IAggregator.sol/IAggregator.json");

abigen!(
    Create2Deployer,
    "$OUT_DIR/ICreate2Deployer.sol/ICreate2Deployer.json"
);

abigen!(EntryPoint, "$OUT_DIR/IEntryPoint.sol/IEntryPoint.json");

abigen!(Paymaster, "$OUT_DIR/IPaymaster.sol/IPaymaster.json");

abigen!(
    StakeManager,
    "$OUT_DIR/IStakeManager.sol/IStakeManager.json"
);

abigen!(Wallet, "$OUT_DIR/IAccount.sol/IAccount.json");

abigen!(
    UserOperation,
    "$OUT_DIR/UserOperation.sol/UserOperationLib.json"
);
