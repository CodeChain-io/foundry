/**
 * An Invoice is used to know whether a transaction or a parcel succeeded or
 * failed.
 */
export class Invoice {
    readonly success: boolean;

    /**
     * @param success Whether a transaction or a parcel succeeded or failed.
     */
    constructor(success: boolean) {
        this.success = !!success;
    }

    // FIXME: any
    /**
     * Create an Invoice from an Invoice JSON object.
     * @param data An Invoice JSON object.
     * @returns An Invoice.
     */
    static fromJSON(data: any) {
        return new this(data === "Success");
    }

    /**
     * Convert to an Invoice JSON object.
     * @returns An Invoice JSON object.
     */
    toJSON() {
        return this.success ? "Success" : "Failed";
    }
}
