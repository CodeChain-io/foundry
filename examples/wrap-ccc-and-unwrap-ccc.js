const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";
const ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

(async () => {
    const address = await sdk.key.createAssetTransferAddress({
        type: "P2PKHBurn"
    });
    const quantity = 100;

    const balanceStart = await sdk.rpc.chain.getBalance(ACCOUNT_ADDRESS);

    // Wrap 100 CCC into the wrapped CCC asset type.
    const wrapCCC = sdk.core.createWrapCCCTransaction({
        shardId: 0,
        recipient: address,
        quantity,
        payer: ACCOUNT_ADDRESS
    });
    const wrapCCCSignedHash = await sdk.rpc.chain.sendTransaction(wrapCCC, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const wrapCCCInvoice = await sdk.rpc.chain.getInvoice(wrapCCCSignedHash, {
        // Wait up to 120 seconds to get the invoice.
        timeout: 120 * 1000
    });
    if (!wrapCCCInvoice) {
        throw Error(`WrapCCC failed: ${JSON.stringify(wrapCCCInvoice.error)}`);
    }
    const balanceAfterWrapCCC = await sdk.rpc.chain.getBalance(ACCOUNT_ADDRESS);
    console.log("Wrap finish");

    // Unwrap the wrapped CCC asset created before.
    const unwrapCCCTx = sdk.core.createUnwrapCCCTransaction({
        burn: wrapCCC.getAsset() // After sendTransaction, the fee and seq field of wrapCCC is filled.
    });
    await sdk.key.signTransactionBurn(unwrapCCCTx, 0);
    const hash = await sdk.rpc.chain.sendTransaction(unwrapCCCTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const unwrapCCCTxInvoice = await sdk.rpc.chain.getInvoice(hash, {
        // Wait up to 120 seconds to get the invoice.
        timeout: 120 * 1000
    });
    if (!unwrapCCCTxInvoice) {
        throw Error(
            `AssetUnwrapCCCTransaction failed: ${JSON.stringify(
                unwrapCCCTxInvoice.error
            )}`
        );
    }
    const balanceAfterUnwrapCCC = await sdk.rpc.chain.getBalance(
        ACCOUNT_ADDRESS
    );

    console.log(balanceStart.toString());
    console.log(balanceAfterWrapCCC.toString());
    console.log(balanceAfterUnwrapCCC.toString());
})().catch(console.error);
