"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/**
 * An Invoice is used to know whether a transaction or a parcel succeeded or
 * failed.
 */
class Invoice {
    /**
     * Create an Invoice from an Invoice JSON object.
     * @param data An Invoice JSON object.
     * @returns An Invoice.
     */
    static fromJSON(data) {
        const { success, error } = data;
        return new Invoice(success, error);
    }
    /**
     * @param success Whether a transaction or a parcel succeeded or failed.
     * @param error.type The type of the error.
     * @param error.content An explanation of the error.
     */
    constructor(success, error) {
        this.success = !!success;
        this.error = error;
    }
    /**
     * Convert to an Invoice JSON object.
     * @returns An Invoice JSON object.
     */
    toJSON() {
        const { success, error } = this;
        return {
            success,
            error
        };
    }
}
exports.Invoice = Invoice;
