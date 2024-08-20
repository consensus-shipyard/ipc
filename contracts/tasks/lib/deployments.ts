import * as fs from 'fs'
import {HardhatRuntimeEnvironment} from 'hardhat/types'
import {Contract} from 'ethers'
import {DeployResult} from 'hardhat-deploy/types'
import {selectors} from "./selectors";
import _ = require('lodash');

export enum FacetCutAction {
    Add,
    Replace,
    Remove,
}

export type FacetCut = Facet & {
    action: FacetCutAction
}

export type Facet = {
    facetAddress: string
    functionSelectors: string[]
}

export class Deployments {
    private readonly _contracts: { [key: string]: Contract }
    private readonly _deployResults?: { [key: string]: DeployResult }

    private constructor(
        contracts: { [key: string]: Contract },
        deployResults?: { [key: string]: DeployResult },
    ) {
        this._contracts = contracts
        this._deployResults = deployResults
    }

    private static async resolveContracts(
        hre: HardhatRuntimeEnvironment,
        contractNames: string[],
    ): Promise<{ [key: string]: Contract }> {
        return Object.fromEntries(
            await Promise.all(
                contractNames.map(async (name) => {
                    const {address} = await hre.deployments.get(name)
                    return [name, await hre.ethers.getContractAt(name, address)]
                }),
            ),
        )
    }

    public static async resolve(
        hre: HardhatRuntimeEnvironment,
        ...contractNames: string[]
    ): Promise<Deployments> {
        return new Deployments(
            await Deployments.resolveContracts(hre, contractNames),
        )
    }

    public static async deploy(
        hre: HardhatRuntimeEnvironment,
        deployer: string,
        ...contracts: { name: string; args?: any; libraries?: string[] }[]
    ): Promise<Deployments> {
        const results = {}
        for (const contract of contracts) {
            console.log(`Deploying ${contract.name}...`)
            const libraries = await Deployments.resolve(
                hre,
                ...(contract.libraries || []),
            )

            const result = await hre.deployments.deploy(contract.name, {
                from: deployer,
                log: true,
                args: contract.args,
                libraries: libraries.addresses,
                waitConfirmations: 1,
            })
            results[contract.name] = result
            console.log(`${contract.name} deployed at ${result.address}`)
        }
        return new Deployments(
            await Deployments.resolveContracts(hre, Object.keys(results)),
            results,
        )
    }

    public asFacetCuts(): FacetCut[] {
        return Object.values(this._contracts).map((contract) => ({
            facetAddress: contract.address,
            action: FacetCutAction.Add,
            functionSelectors: selectors(contract),
        }))
    }

    public join(other: Deployments): Deployments {
        return new Deployments(
            Object.assign({}, this._contracts, other._contracts),
            Object.assign({}, this._deployResults, other._deployResults),
        )
    }

    get addresses(): { [key: string]: string } {
        return _.mapValues(
            this.contracts,
            (contract: Contract) => contract.address,
        )
    }

    get contracts(): { [key: string]: Contract } {
        return this._contracts
    }

    get results(): { [key: string]: DeployResult } {
        return this._deployResults
    }
}
