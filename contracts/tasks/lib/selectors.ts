import {Interface} from "ethers/lib/utils";

interface ContractLike {
    interface: Interface;
}

export function selectors(contract: ContractLike) {
    return Object.keys(contract.interface.functions)
        .filter((sig) => sig !== 'init(bytes)')
        .map((sig) => contract.interface.getSighash(sig))
}