import { SDK } from "../";

import { SERVER_URL } from "./helper";

const sdk = new SDK({ server: SERVER_URL });

test("shareSecret", async () => {
    const result = await sdk.rpc.network.shareSecret(
        "0x24df02abcd4e984e90253dc344e89b8431bbb319c66643bfef566dfdf46ec6bc",
        "127.0.0.1",
        3486
    );
    expect(result).toBe(null);
});
