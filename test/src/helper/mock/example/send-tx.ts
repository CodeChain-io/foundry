import { Mock } from "..";
import * as SDK from "../../../sdk";

async function sendTransaction() {
    const mock = new Mock("0.0.0.0", 3485, "tc");
    mock.setLog();
    await mock.establish();

    const sdk = new SDK({
        server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
        networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
    });
    const ACCOUNT_SECRET =
        process.env.ACCOUNT_SECRET ||
        "9af28f6fd6a1170dbee2cb8c34abab0408e6d811d212cdcde23f72473eb0d97ad7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c";
    const unsigned = sdk.core.createPayTransaction({
        recipient: "tccqruq09sfgax77nj4gukjcuq69uzeyv0jcs7vzngg",
        amount: 10000
    });
    const signed = unsigned.sign({
        secret: ACCOUNT_SECRET,
        fee: 10,
        nonce: 0
    });

    await mock.sendEncodedTransaction([signed.toEncodeObject()]);

    await mock.end();
}
sendTransaction();
