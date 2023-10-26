// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "elliptic-curve-solidity/contracts/EllipticCurve.sol";

library TestUtils {
    uint256 public constant GX = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;
    uint256 public constant GY = 0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8;
    uint256 public constant AA = 0;
    uint256 public constant BB = 7;
    uint256 public constant PP = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;

    function derivePubKey(uint256 privKey) external pure returns (uint256, uint256) {
        return EllipticCurve.ecMul(privKey, GX, GY, AA, PP);
    }

    function derivePubKeyBytes(uint256 privKey) external pure returns (bytes memory) {
        (uint256 pubKeyX, uint256 pubKeyY) = EllipticCurve.ecMul(privKey, GX, GY, AA, PP);
        return abi.encode(pubKeyX, pubKeyY);
    }

    function deriveValidatorPubKeyBytes(uint256 privKey) external pure returns (bytes memory) {
        (uint256 pubKeyX, uint256 pubKeyY) = EllipticCurve.ecMul(privKey, GX, GY, AA, PP);

        // https://github.com/ethereum/eth-keys/blob/master/README.md#keyapipublickeypublic_key_bytes

        return abi.encodePacked(uint8(0x4), pubKeyX, pubKeyY);
    }

    function generateSelectors(Vm vm, string memory facetName) internal returns (bytes4[] memory facetSelectors) {
        string[] memory inputs = new string[](3);
        inputs[0] = "python3";
        inputs[1] = "scripts/python/get_selectors.py";
        inputs[2] = facetName;

        bytes memory res = vm.ffi(inputs);
        facetSelectors = abi.decode(res, (bytes4[]));
    }

    function getThreeValidators(
        Vm vm
    ) internal returns (uint256[] memory validatorKeys, address[] memory addresses, uint256[] memory weights) {
        validatorKeys = new uint256[](3);
        validatorKeys[0] = 100;
        validatorKeys[1] = 200;
        validatorKeys[2] = 300;

        addresses = new address[](3);
        addresses[0] = vm.addr(validatorKeys[0]);
        addresses[1] = vm.addr(validatorKeys[1]);
        addresses[2] = vm.addr(validatorKeys[2]);

        weights = new uint256[](3);
        vm.deal(vm.addr(validatorKeys[0]), 1);
        vm.deal(vm.addr(validatorKeys[1]), 1);
        vm.deal(vm.addr(validatorKeys[2]), 1);

        weights = new uint256[](3);
        weights[0] = 100;
        weights[1] = 101;
        weights[2] = 102;
    }

    function deriveValidatorAddress(uint8 seq) internal pure returns (address addr, bytes memory data) {
        data = new bytes(65);
        data[1] = bytes1(seq);

        // use data[1:] for the hash
        bytes memory dataSubset = new bytes(data.length - 1);
        for (uint i = 1; i < data.length; i++) {
            dataSubset[i - 1] = data[i];
        }

        addr = address(uint160(uint256(keccak256(dataSubset))));
    }

    function derivePubKey(uint8 seq) internal pure returns (address addr, bytes memory data) {
        data = new bytes(65);
        data[1] = bytes1(seq);

        // use data[1:] for the hash
        bytes memory dataSubset = new bytes(data.length - 1);
        for (uint i = 1; i < data.length; i++) {
            dataSubset[i - 1] = data[i];
        }

        addr = address(uint160(uint256(keccak256(dataSubset))));
    }

    function ensureBytesEqual(bytes memory _a, bytes memory _b) internal pure {
        require(_a.length == _b.length, "bytes len not equal");
        require(keccak256(_a) == keccak256(_b), "bytes not equal");
    }
}
