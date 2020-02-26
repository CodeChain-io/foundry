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
    faucetAddress: string;
}

export function getConfig(): Config {
    return {
        chainA: {
            rpcURL: getEnv("CHAIN_A_RPC_URL"),
            networkId: getEnv("CHAIN_A_NETWORK_ID"),
            faucetAddress: getEnv("CHAIN_A_FAUCET_ADDRESS")
        },
        chainB: {
            rpcURL: getEnv("CHAIN_B_RPC_URL"),
            networkId: getEnv("CHAIN_B_NETWORK_ID"),
            faucetAddress: getEnv("CHAIN_B_FAUCET_ADDRESS")
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
