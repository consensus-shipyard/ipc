import {Contract} from "ethers";

export function selectors(contract: Contract) {
    return Object.keys(contract.interface.functions)
        .filter((sig) => sig !== 'init(bytes)')
        .map((sig) => contract.interface.getSighash(sig))
}