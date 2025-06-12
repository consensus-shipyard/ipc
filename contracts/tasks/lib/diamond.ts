import {Contract} from 'ethers'
import {HardhatRuntimeEnvironment} from 'hardhat/types'
import {IDiamondLoupe} from '../../typechain'
import {Deployments, FacetCut} from './deployments'
import {selectors} from "./selectors";
import _ = require('lodash');

enum FacetCutAction {
    Add = 0,
    Replace = 1,
    Remove = 2,
}

type Selectors = { [selector: string]: string }

export class Diamond {
    public readonly contract: Contract
    private hre: HardhatRuntimeEnvironment

    constructor(hre: HardhatRuntimeEnvironment, contract: Contract) {
        this.hre = hre
        this.contract = contract
    }

    /**
     * Computes the facet cuts needed to upgrade the diamond to the target.
     * @param target The end state we desire.
     */
    public async computeFacetCuts(target: Deployments): Promise<FacetCut[]> {
        // These objects will hold selector => facet address mappings.
        const loupe = (await this.hre.ethers.getContractAt(
            'DiamondLoupeFacet',
            this.contract.address,
        )) as IDiamondLoupe

        // Current selectors (selector => address).
        const current: Selectors = {}
        for (const facet of await loupe.facets()) {
            for (const selector of facet.functionSelectors) {
                current[selector] = facet.facetAddress
            }
        }

        // Needed selectors (selector => address).
        const needed: Selectors = {}
        for (const contract of Object.values(target.contracts)) {
            for (const selector of selectors(contract)) {
                needed[selector] = contract.address
            }
        }

        // Generate the facet cuts.
        const facetCuts: FacetCut[] = []
        facetCuts.push(...this.computeRemoved(current, needed))
        facetCuts.push(...this.computeAdded(current, needed))
        facetCuts.push(...this.computeReplaced(current, needed))

        return facetCuts
    }

    /**
     * Computes the removed facet cuts.
     */
    private computeRemoved(current: Selectors, needed: Selectors): FacetCut[] {
        const selectorsToRemove = Object.keys(current).filter(
            (selector) => !needed[selector],
        )
        if (selectorsToRemove.length === 0) {
            return []
        }
        return [
            {
                facetAddress: '0x0000000000000000000000000000000000000000',
                action: FacetCutAction.Remove,
                functionSelectors: selectorsToRemove,
            },
        ]
    }

    /**
     * Computes the added facet cuts.
     */
    private computeAdded(current: Selectors, needed: Selectors): FacetCut[] {
        const selectorsToAdd = Object.keys(needed).filter(
            (selector) => !current[selector],
        )

        const addressToSelectors = _.groupBy(selectorsToAdd, (selector) => needed[selector])

        return Object.entries(addressToSelectors).map(([facetAddress, functionSelectors]) => ({
            facetAddress,
            action: FacetCutAction.Add,
            functionSelectors,
        }))
    }

    /**
     * Computes the replaced facet cuts.
     */
    private computeReplaced(current: Selectors, needed: Selectors): FacetCut[] {
        const selectorsToReplace = Object.keys(needed).filter(
            (selector) =>
                current[selector] && current[selector] !== needed[selector],
        )
        const addressToSelectors = _.groupBy(selectorsToReplace, (selector) => needed[selector])

        return Object.entries(addressToSelectors).map(([facetAddress, functionSelectors]) => ({
            facetAddress,
            action: FacetCutAction.Replace,
            functionSelectors,
        }))
    }
}
