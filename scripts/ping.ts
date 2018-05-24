import { SDK } from "../src";
import * as assert from "assert";

const sdk = new SDK("http://localhost:8080");

sdk.ping().then(() => {
    console.log("Pong");
}).catch((err) => {
    assert.fail(err);
})
