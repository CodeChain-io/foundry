import { SDK } from "../";

const SERVER_URL = "http://localhost:8080";

test('ping', async () => {
    new SDK(SERVER_URL).ping().then(res => {
        expect(res).toBe("pong");
    });
}, 100);
