const {
    Application,
    genesisBlockDevnet,
    configDevnet,
    HTTPAPIPlugin,
    utils,
} = require("lisk-sdk");
// 2, import SRS module & plugins
const { SRSModule } = require('./src_module/index');
const { SRSAPIPlugin } = require('./plugins/srs_api_plugin');
const { SRSDataPlugin } = require("./plugins/srs_data_plugin");

// 3. update the genesis block accounts to include SRS module attributes
genesisBlockDevnet.header.timestamp = 1605699440;
genesisBlockDevnet.header.asset.accounts = genesisBlockDevnet.header.asset.accounts.map(
    (account) =>
        utils.objects.mergeDeep({}, account, {
            srs: {
                config: {
                    friends: [],
                    recoveryThreshold: 0,
                    delayPeriod: 0,
                },
                status: {
                    active: false,
                    vouchList: [],
                    created: 0,
                    deposit: BigInt(0),
                    rescuer: Buffer.from(''),
                },
            },
        }),
);

// 4 update application config to include unique label
// and communityIdentifier to mitigate transaction replay
const appConfig = utils.objects.mergeDeep({}, configDevnet, {
    label: 'srs-app',
    genesisConfig: { communityIdentifier: 'SRS' },
    logger: {
        consoleLogLevel: 'info',
    },
    rpc: {
        enable: true,
        mode: 'ws',
        port: 8888,
    },
});

// 5. Initialize the application with genesis block and application config
const app = Application.defaultApplication(genesisBlockDevnet, appConfig);

// 6. Register custom SRS Module and Plugins
app.registerModule(SRSModule);
app.registerPlugin(HTTPAPIPlugin);
app.registerPlugin(SRSAPIPlugin);
app.registerPlugin(SRSDataPlugin);

app
    .run()
    .then(() => app.logger.info("SRS Blockchain running...."))
    .catch(() => {
        console.error("Faced error in application..", error);
    });
