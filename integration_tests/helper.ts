import { SDK } from "../";

export const CODECHAIN_NETWORK_ID = process.env.CODECHAIN_NETWORK_ID || "tc";
export const SERVER_URL =
    process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
export const sdk = new SDK({
    server: SERVER_URL,
    keyStoreType: "memory",
    networkId: CODECHAIN_NETWORK_ID
});

export const ACCOUNT_SECRET =
    process.env.ACCOUNT_SECRET ||
    "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
export const ACCOUNT_ID =
    process.env.ACCOUNT_ID ||
    sdk.util.getAccountIdFromPrivate(ACCOUNT_SECRET).toString(); // "0xa6594b7196808d161b6fb137e781abbc251385d9"
export const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    sdk.key.classes.PlatformAddress.fromAccountId(ACCOUNT_ID).toString(); // "tccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9my9a2k78"
export const ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

export const sendTransactions = async ({ transactions }: any) => {
    const parcel = sdk.core.createAssetTransactionGroupParcel({
        transactions
    });
    const signedParcel = parcel.sign({
        secret: ACCOUNT_SECRET,
        nonce: await sdk.rpc.chain.getNonce(ACCOUNT_ADDRESS),
        fee: 10
    });
    const parcelHash = await sdk.rpc.chain.sendSignedParcel(signedParcel);
    return {
        parcelHash
    };
};

export const mintAsset = async ({
    metadata,
    amount,
    lockScriptHash,
    registrar
}: any) => {
    const assetScheme = sdk.core.createAssetScheme({
        shardId: 0,
        worldId: 0,
        metadata,
        amount,
        registrar
    });
    const assetAddress = sdk.key.classes.AssetTransferAddress.fromTypeAndPayload(
        0,
        lockScriptHash
    );
    const assetMintTransaction = assetScheme.createMintTransaction({
        recipient: assetAddress,
        nonce: Math.floor(Math.random() * 1000000000)
    });
    return {
        ...(await sendTransactions({ transactions: [assetMintTransaction] })),
        assetMintTransaction
    };
};

export const payment = async (params?: { inc_nonce?: number }) => {
    const { inc_nonce = 0 } = params || {};
    let nonce = await sdk.rpc.chain.getNonce(ACCOUNT_ADDRESS);
    for (let i = 0; i < inc_nonce; i++) {
        nonce = nonce.increase();
    }
    const p = sdk.core
        .createPaymentParcel({
            amount: 10,
            recipient: ACCOUNT_ADDRESS
        })
        .sign({
            secret: ACCOUNT_SECRET,
            fee: 10,
            nonce
        });
    return await sdk.rpc.chain.sendSignedParcel(p);
};
