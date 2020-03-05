export interface Config {
    chainA: FoundryChainConfig;
    chainB: FoundryChainConfig;
}

interface FoundryChainConfig {
    /**
     * Foundry RPC URL
     * ex) http://localhost:8080
     */
    rpcURL: string;
    networkId: string;
    relayerAddress: string;
    scenarioAddress: string;
    counterpartyClientId: string;
    counterpartyConnectionId: string;
    counterpartyChannelId: string;
    keystorePath: string;
}

export function getConfig(): Config {
    return {
        chainA: {
            rpcURL: getEnv("CHAIN_A_RPC_URL"),
            networkId: getEnv("CHAIN_A_NETWORK_ID"),
            relayerAddress: getEnv("CHAIN_A_RELAYER_ADDRESS"),
            scenarioAddress: getEnv("CHAIN_A_SCENARIO_ADDRESS"),
            counterpartyClientId: getEnv("CHAIN_A_COUNTERPARTY_CLIENT_ID"),
            counterpartyConnectionId: getEnv(
                "CHAIN_A_COUNTERPARTY_CONNECTION_ID"
            ),
            counterpartyChannelId: getEnv("CHAIN_A_COUNTERPARTY_CHANNEL_ID"),
            keystorePath: "./chainA/keystore.db"
        },
        chainB: {
            rpcURL: getEnv("CHAIN_B_RPC_URL"),
            networkId: getEnv("CHAIN_B_NETWORK_ID"),
            relayerAddress: getEnv("CHAIN_B_RELAYER_ADDRESS"),
            scenarioAddress: getEnv("CHAIN_B_SCENARIO_ADDRESS"),
            counterpartyClientId: getEnv("CHAIN_B_COUNTERPARTY_CLIENT_ID"),
            counterpartyConnectionId: getEnv(
                "CHAIN_B_COUNTERPARTY_CONNECTION_ID"
            ),
            counterpartyChannelId: getEnv("CHAIN_B_COUNTERPARTY_CHANNEL_ID"),
            keystorePath: "./chainB/keystore.db"
        }
    };
}

function getEnv(key: string): string {
    const result = process.env[key];
    if (result) {
        return result;
    } else {
        throw new Error(`Environment variable ${key} is not set`);
    }
}
