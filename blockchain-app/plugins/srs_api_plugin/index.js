const { BasePlugin } = require("lisk-sdk");
const pJSON = require("../../package.json");

// 1, plugins can be a daemon/HTTP/Websocket service for off chain processing
class SRSAPIPlugin extends BasePlugin {
    _server = undefined;
    _app = undefined;
    _channel = undefined;

    static get alias() {
        return 'SRSHttpApi';
    }

    static get info() {
        return {
            author: pJSON.author,
            version: pJSON.version,
            name: pJSON.name,
        };
    }

    get defaults() {
        return {};
    }

    get events() {
        return [];
    }

    get actions() {
        return {};
    }
}

module.exports = { SRSAPIPlugin };

