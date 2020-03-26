import { Address, H256, SignedTransaction, Transaction, U64 } from "../classes";
import { SignedTransactionJSON } from "../SignedTransaction";
import { ChangeParams } from "./ChangeParams";
import { DelegateCCS } from "./DelegateCCS";
import { Pay } from "./Pay";
import { Redelegate } from "./Redelegate";
import { ReportDoubleVote } from "./ReportDoubleVote";
import { Revoke } from "./Revoke";
import { SelfNominate } from "./SelfNominate";
import { TransferCCS } from "./TransferCCS";

export function fromJSONToTransaction(result: any): Transaction {
    const { seq, fee, networkId, action } = result;
    let tx;
    switch (action.type) {
        case "pay": {
            const receiver = Address.ensure(action.receiver);
            const quantity = new U64(action.quantity);
            tx = new Pay(receiver, quantity, networkId);
            break;
        }
        case "delegateCCS": {
            const address = Address.ensure(action.address);
            const quantity = new U64(action.quantity);
            tx = new DelegateCCS(address, quantity, networkId);
            break;
        }
        case "transferCCS": {
            const address = Address.ensure(action.address);
            const quantity = new U64(action.quantity);
            tx = new TransferCCS(address, quantity, networkId);
            break;
        }
        case "revoke": {
            const address = Address.ensure(action.address);
            const quantity = new U64(action.quantity);
            tx = new Revoke(address, quantity, networkId);
            break;
        }
        case "redelegate": {
            const prevDelegator = Address.ensure(action.prevDelegator);
            const nextDelegator = Address.ensure(action.nextDelegator);
            const quantity = new U64(action.quantity);
            tx = new Redelegate(
                prevDelegator,
                nextDelegator,
                quantity,
                networkId
            );
            break;
        }
        case "selfNominate": {
            const deposit = new U64(action.deposit);
            const metadata = Buffer.from(action.metadata);
            tx = new SelfNominate(deposit, metadata, networkId);
            break;
        }
        case "reportDoubleVote": {
            const message1 = Buffer.from(action.message1);
            const message2 = Buffer.from(action.message2);
            tx = new ReportDoubleVote(message1, message2, networkId);
            break;
        }
        case "changeParams": {
            const metadataSeq = new U64(action.metadataSeq);
            const params = action.params;
            const approvals = action.approvals;
            tx = new ChangeParams(metadataSeq, params, approvals, networkId);
            break;
        }
        default:
            throw Error(`Unexpected action: ${action}`);
    }
    if (seq != null) {
        tx.setSeq(seq);
    }
    if (fee != null) {
        tx.setFee(fee);
    }
    return tx;
}

/**
 * Create a SignedTransaction from a SignedTransaction JSON object.
 * @param data A SignedTransaction JSON object.
 * @returns A SignedTransaction.
 */
export function fromJSONToSignedTransaction(data: SignedTransactionJSON) {
    const {
        sig,
        signerPublic,
        blockNumber,
        blockHash,
        transactionIndex
    } = data;
    if (typeof sig !== "string") {
        throw Error("Unexpected type of sig");
    }
    if (blockNumber != null && blockHash != null && transactionIndex != null) {
        return new SignedTransaction(
            fromJSONToTransaction(data),
            sig,
            signerPublic,
            blockNumber,
            new H256(blockHash),
            transactionIndex
        );
    } else {
        return new SignedTransaction(
            fromJSONToTransaction(data),
            sig,
            signerPublic
        );
    }
}
