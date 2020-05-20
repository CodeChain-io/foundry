import { Mock } from "..";
import { SDK } from "../../../sdk";

async function sendTransaction() {
    const mock = new Mock("0.0.0.0", 3485, "tc");
    mock.setLog();
    await mock.establish();

    const sdk = new SDK({
        networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
    });
    const ACCOUNT_SECRET =
        process.env.ACCOUNT_SECRET ||
        "9af28f6fd6a1170dbee2cb8c34abab0408e6d811d212cdcde23f72473eb0d97ad7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c";
    const unsigned = sdk.core.createPayTransaction({
        recipient: "fys3db1kOrI_rXyaTx9U2_RP-SlNK1q0LRXxYeQGBI1av35drZQtc0",
        quantity: 10000
    });
    const signed = unsigned.sign({
        secret: ACCOUNT_SECRET,
        fee: 10,
        seq: 0
    });

    await mock.sendEncodedTransaction([signed.toEncodeObject()]);

    await mock.end();
}
sendTransaction();
