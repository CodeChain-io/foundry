const SDK = require("codechain-sdk");
const sdk = new SDK({
    server: "http://52.78.210.78:8080",
    networkId: "sc"
});

const parcelSender = process.env.CODECHAIN_SALUKI_ACCOUNT;
if (!sdk.core.classes.PlatformAddress.check(parcelSender)) {
    throw Error(
        "The environment variable CODECHAIN_SALUKI_ACCOUNT must be a valid platform address for Saluki. For example: sccqz8hyh3560xwpykm9u8en5k2jcwcueq6ncvg2dvy"
    );
}

let lastBlockNumber = 9706;
setInterval(async () => {
    const bestBlockNumber = await sdk.rpc.chain.getBestBlockNumber();
    if (bestBlockNumber <= lastBlockNumber) {
        return;
    }
    const block = await sdk.rpc.chain.getBlock(++lastBlockNumber);

    const transactions = extractTransactions(block);
    console.log(
        `Total ${transactions.length} transactions in block ${block.number}`
    );
    const addresses = extractAddresses(transactions);
    console.log(`Found ${addresses.length} addresses`);
    if (addresses.length > 0) {
        sendCoins(addresses);
    }
}, 5000);

const { AssetTransactionGroup } = sdk.core.classes;

function extractTransactions(block) {
    return block.parcels
        .filter(p => p.unsigned.action instanceof AssetTransactionGroup)
        .map(p => p.unsigned.action.transactions)
        .reduce((prev, curr) => [...prev, ...curr], []);
}

const {
    AssetTransferAddress,
    AssetMintTransaction,
    AssetTransferTransaction
} = sdk.core.classes;

function extractAddresses(transactions) {
    return transactions
        .map(t => {
            if (t instanceof AssetMintTransaction) {
                if (isP2PKHScript(t.output)) {
                    const publicKeyHash = t.output.parameters[0].toString(
                        "hex"
                    );
                    return [
                        AssetTransferAddress.fromTypeAndPayload(
                            1,
                            publicKeyHash,
                            {
                                networkId: "sc"
                            }
                        ).value
                    ];
                } else {
                    return [];
                }
            } else if (t instanceof AssetTransferTransaction) {
                return t.outputs
                    .filter(output => isP2PKHScript(output))
                    .map(output => {
                        const publicKeyHash = output.parameters[0].toString(
                            "hex"
                        );
                        return AssetTransferAddress.fromTypeAndPayload(
                            1,
                            publicKeyHash,
                            {
                                networkId: "sc"
                            }
                        ).value;
                    });
            } else {
                return [];
            }
        })
        .reduce((prev, curr) => [...prev, ...curr], []);
}

function isP2PKHScript(output) {
    const P2pkhLockScriptHash =
        "f42a65ea518ba236c08b261c34af0521fa3cd1aa505e1c18980919cb8945f8f3";
    const P2pkhBurnLockScriptHash =
        "41a872156efc1dbd45a85b49896e9349a4e8f3fb1b8f3ed38d5e13ef675bcd5a";
    return (
        output.parameters.length === 1 &&
        (output.lockScriptHash.value === P2pkhLockScriptHash ||
            output.lockScriptHash.value === P2pkhBurnLockScriptHash)
    );
}

const assetOwner = /* your asset address here */ notImplemented();
let lastTransactionHash = /* the hash of either mint transaction or transfer transaction */ notImplemented();
async function sendCoins(recipients) {
    console.log(`Send to ${recipients}`);

    const isAssetSpent = await sdk.rpc.chain.isAssetSpent(
        lastTransactionHash,
        0,
        0
    );
    if (isAssetSpent === null) {
        throw Error(`No such asset for tx(0x${lastTransactionHash})`);
    } else if (isAssetSpent === true) {
        throw Error(
            `The asset was spent already. Check lastTransactionHash(${lastTransactionHash})`
        );
    }

    const keyStore = await sdk.key.createLocalKeyStore();
    const asset = await sdk.rpc.chain.getAsset(lastTransactionHash, 0);
    const transferTx = sdk.core
        .createAssetTransferTransaction()
        .addInputs(asset)
        .addOutputs(
            {
                recipient: assetOwner,
                amount: asset.amount - recipients.length,
                assetType: asset.assetType
            },
            ...recipients.map(recipient => ({
                recipient,
                amount: 1,
                assetType: asset.assetType
            }))
        );
    await sdk.key.signTransactionInput(transferTx, 0, {
        keyStore
    });

    const parcel = sdk.core.createAssetTransactionGroupParcel({
        transactions: [transferTx]
    });
    const signedParcel = await sdk.key.signParcel(parcel, {
        keyStore,
        account: parcelSender,
        fee: 10,
        nonce: await sdk.rpc.chain.getNonce(parcelSender)
    });
    const parcelHash = await sdk.rpc.chain.sendSignedParcel(signedParcel);
    console.log(
        `https://saluki.codechain.io/explorer/parcel/0x${parcelHash.value}\n`
    );
    lastTransactionHash = transferTx.hash().value;
}

function notImplemented() {
    throw Error();
}
