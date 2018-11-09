import { AccountRpc } from "./account";
import { ChainRpc } from "./chain";
import { DevelRpc } from "./devel";
import { NetworkRpc } from "./network";
import { NodeRpc } from "./node";
export declare class Rpc {
    /**
     * RPC module for retrieving the node info.
     */
    node: NodeRpc;
    /**
     * RPC module for accessing the blockchain.
     */
    chain: ChainRpc;
    /**
     * RPC module for configuring P2P networking of the node.
     */
    network: NetworkRpc;
    /**
     * RPC module for account management and signing
     */
    account: AccountRpc;
    devel: DevelRpc;
    private client;
    /**
     * @param params.server HTTP RPC server address.
     * @param params.options.parcelSigner The default account to sign the parcel
     * @param params.options.parcelFee The default amount for the parcel fee
     */
    constructor(params: {
        server: string;
        options?: {
            parcelSigner?: string;
            parcelFee?: number;
        };
    });
    sendRpcRequest: (name: string, params: any[]) => Promise<any>;
}
