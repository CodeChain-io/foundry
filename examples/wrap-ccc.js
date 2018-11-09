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
        type: "P2PKH"
    });
    const amount = 100;

    const balanceBefore = await sdk.rpc.chain.getBalance(ACCOUNT_ADDRESS);

    // Wrap 100 CCC into the Wrapped CCC asset type.
    const wrapCCCParcel = sdk.core.createWrapCCCParcel({
        shardId: 0,
        recipient: address,
        amount
    });
    const parcelHash = await sdk.rpc.chain.sendParcel(wrapCCCParcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const invoice = await sdk.rpc.chain.getParcelInvoice(parcelHash, {
        // Wait up to 120 seconds to get the invoice.
        timeout: 120 * 1000
    });
    console.log(invoice); // [{ success: true }]

    // Difference should be sdk.rpc.chain.parcelFee + amount
    const balanceAfter = await sdk.rpc.chain.getBalance(ACCOUNT_ADDRESS);
    console.log(balanceBefore.toString());
    console.log(balanceAfter.toString());
    console.log(sdk.rpc.chain.parcelFee + amount);
})().catch(console.error);
