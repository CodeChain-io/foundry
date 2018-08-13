import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

const secret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
const accountId = sdk.util.getAccountIdFromPrivate(secret);
const address = sdk.key.classes.PlatformAddress.fromAccountId(accountId);

export const sendTransactions = async ({ transactions }) => {
    const parcel = sdk.core.createChangeShardStateParcel({
        transactions,
    });
    const signedParcel = parcel.sign({
        secret,
        nonce: await sdk.rpc.chain.getNonce(address),
        fee: 10
    });
    const parcelHash = await sdk.rpc.chain.sendSignedParcel(signedParcel);
    return {
        parcelHash
    };
};

export const mintAsset = async ({ metadata, amount, lockScriptHash, registrar }) => {
    const assetScheme = sdk.core.createAssetScheme({ metadata, amount, registrar });
    const assetAddress = sdk.key.classes.AssetTransferAddress.fromLockScriptHash(lockScriptHash);
    const assetMintTransaction = assetScheme.createMintTransaction({ recipient: assetAddress, nonce: Math.floor(Math.random() * 1000000000) });
    return {
        ...await sendTransactions({ transactions: [assetMintTransaction] }),
        assetMintTransaction
    };
};

export const payment = async (params?: { inc_nonce?: number }) => {
    const { inc_nonce = 0 } = params || {};
    let nonce = await sdk.rpc.chain.getNonce(address);
    for (let i = 0; i < inc_nonce; i++) {
        nonce = nonce.increase();
    }
    const p = sdk.core.createPaymentParcel({
        amount: 10,
        recipient: address,
    }).sign({
        secret,
        fee: 10,
        nonce
    });
    return await sdk.rpc.chain.sendSignedParcel(p);
};
