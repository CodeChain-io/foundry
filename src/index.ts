const jayson = require('jayson');

class SDK {
    private client;

    constructor(httpUrl) {
        this.client = jayson.client.http(httpUrl);
    }

    ping() {
        return new Promise((resolve, reject) => {
            this.client.request("ping", [], (err, res) => {
                if (err) {
                    return reject(err);
                }
                resolve(res.result);
            });
        });
    }
}

module.exports = SDK;
