// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import {MultisignatureChecker} from "../src/lib/LibMultisignatureChecker.sol";
import {ECDSA} from "openzeppelin-contracts/utils/cryptography/ECDSA.sol";

contract SignerTest is StdInvariant, Test {
    function testBasicSignerInterface() public pure {
        uint256 PRIVATE_KEY = 1000;
        address signer = vm.addr(PRIVATE_KEY);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(PRIVATE_KEY, hash);
        bytes memory signature = abi.encodePacked(r, s, v);

        address s1 = ECDSA.recover(hash, signature);
        require(s1 == signer, "s1 == signer");
    }

    function testMultiSignatureChecker_Weighted_OneSignature() public pure {
        uint256 PRIVATE_KEY = 1000;
        address signer = vm.addr(PRIVATE_KEY);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(PRIVATE_KEY, hash);
        bytes memory signatureBytes = abi.encodePacked(r, s, v);

        require(signatureBytes.length == 65, "signatureBytes.length == 65");

        address[] memory signers = new address[](1);
        signers[0] = signer;

        uint256[] memory weights = new uint256[](1);
        weights[0] = 10;

        (bool valid, MultisignatureChecker.Error err) = MultisignatureChecker.isValidWeightedMultiSignature(
            signers,
            weights,
            10,
            hash,
            signatureBytes
        );
        require(valid == true, "valid == true");
        require(err == MultisignatureChecker.Error.Nil, "err == Nil");
    }

    function testMultiSignatureChecker_Weighted_FourSignatures() public pure {
        uint256 PRIVATE_KEY_BASE = 1000;
        address[] memory signers = new address[](4);
        uint256[] memory weights = new uint256[](4);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        bytes memory multisignatureBytes;

        for (uint256 i = 0; i < 4; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(PRIVATE_KEY_BASE + i, hash);
            bytes memory signature = abi.encodePacked(r, s, v);
            signers[i] = vm.addr(PRIVATE_KEY_BASE + i);
            weights[i] = 10;

            multisignatureBytes = bytes.concat(multisignatureBytes, signature);
        }

        require(multisignatureBytes.length == 65 * 4, "multisignatureBytes.length == 65 * 4");

        (bool valid, MultisignatureChecker.Error err) = MultisignatureChecker.isValidWeightedMultiSignature(
            signers,
            weights,
            30,
            hash,
            multisignatureBytes
        );
        require(valid == true, "valid == true");
        require(err == MultisignatureChecker.Error.Nil, "err == Nil");
    }

    function testMultiSignatureChecker_Weighted_InvalidSignaturesLength() public pure {
        bytes32 hash = keccak256(abi.encodePacked("test"));

        address[] memory signers = new address[](1);
        signers[0] = vm.addr(101);

        uint256[] memory weights = new uint256[](1);
        weights[0] = 10;

        (bool valid, MultisignatureChecker.Error err) = MultisignatureChecker.isValidWeightedMultiSignature(
            signers,
            weights,
            10,
            hash,
            abi.encodePacked(hash)
        );
        require(valid == false, "valid == false");
        require(err == MultisignatureChecker.Error.InvalidSignaturesBytes);

        (valid, err) = MultisignatureChecker.isValidWeightedMultiSignature(
            signers,
            weights,
            10,
            hash,
            bytes("1234567890")
        );
        require(valid == false, "valid == false");
        require(err == MultisignatureChecker.Error.InvalidSignaturesBytes);

        bytes memory signature66 = bytes.concat(abi.encodePacked(hash), abi.encodePacked(hash), "1", "1");

        (valid, err) = MultisignatureChecker.isValidWeightedMultiSignature(signers, weights, 10, hash, signature66);
        require(valid == false, "valid == false");
        require(err == MultisignatureChecker.Error.InvalidSignaturesBytes);

        (valid, err) = MultisignatureChecker.isValidWeightedMultiSignature(signers, weights, 10, hash, bytes(""));
        require(valid == false, "valid == false");
        require(err == MultisignatureChecker.Error.InvalidSignaturesBytes, "err == InvalidSignaturesBytes");
    }

    function testMultiSignatureChecker_Weighted_InvalidSignatureInMultisig() public pure {
        uint256 PRIVATE_KEY_BASE = 1000;
        address[] memory signers = new address[](4);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        bytes memory multisignatureBytes;
        bytes32 b;

        uint256[] memory weights = new uint256[](4);

        for (uint256 i = 0; i < 4; i++) {
            (uint8 v, bytes32 r, ) = vm.sign(PRIVATE_KEY_BASE + i, hash);
            bytes memory signature = abi.encodePacked(r, b, v);
            signers[i] = vm.addr(PRIVATE_KEY_BASE + i);
            weights[i] = 10;

            multisignatureBytes = bytes.concat(multisignatureBytes, signature);
        }

        (bool valid, MultisignatureChecker.Error err) = MultisignatureChecker.isValidWeightedMultiSignature(
            signers,
            weights,
            30,
            hash,
            abi.encodePacked(multisignatureBytes)
        );
        require(valid == false, "valid == false");
        require(err == MultisignatureChecker.Error.InvalidSignature, "err == InvalidSignature");
    }

    function testMultiSignatureChecker_Weighted_InvalidSignatureOfSigner() public pure {
        uint256 PRIVATE_KEY_BASE = 1000;
        address[] memory signers = new address[](2);
        uint256[] memory weights = new uint256[](2);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        bytes memory multisignatureBytes;

        for (uint256 i = 0; i < 2; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(PRIVATE_KEY_BASE + i, hash);
            bytes memory signature = abi.encodePacked(r, s, v);
            multisignatureBytes = bytes.concat(multisignatureBytes, signature);
            weights[i] = 10;
        }

        // use invalid keys
        signers[0] = vm.addr(PRIVATE_KEY_BASE + 1);
        signers[1] = vm.addr(PRIVATE_KEY_BASE);

        (bool valid, MultisignatureChecker.Error err) = MultisignatureChecker.isValidWeightedMultiSignature(
            signers,
            weights,
            10,
            hash,
            multisignatureBytes
        );
        require(valid == false, "valid == false");
        require(err == MultisignatureChecker.Error.InvalidSigner, "err == InvalidSigner");
    }

    function testMultiSignatureChecker_Weighted_LessThanThreshold() public pure {
        uint256 PRIVATE_KEY_BASE = 1000;
        address[] memory signers = new address[](2);
        uint256[] memory weights = new uint256[](2);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        bytes memory multisignatureBytes;

        for (uint256 i = 0; i < 2; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(PRIVATE_KEY_BASE + i, hash);
            bytes memory signature = abi.encodePacked(r, s, v);
            multisignatureBytes = bytes.concat(multisignatureBytes, signature);
            weights[i] = 10;
            signers[i] = vm.addr(PRIVATE_KEY_BASE + i);
        }

        (bool valid, MultisignatureChecker.Error err) = MultisignatureChecker.isValidWeightedMultiSignature(
            signers,
            weights,
            100,
            hash,
            multisignatureBytes
        );
        require(valid == false, "valid == false");
        require(err == MultisignatureChecker.Error.WeightsSumLessThanThreshold, "err == WeightsSumLessThanThreshold");
    }

    function testMultiSignatureChecker_Weighted_InvalidNumberOfWeights() public pure {
        uint256 PRIVATE_KEY_BASE = 1000;
        address[] memory signers = new address[](2);
        uint256[] memory weights = new uint256[](1);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        bytes memory multisignatureBytes;

        for (uint256 i = 0; i < 2; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(PRIVATE_KEY_BASE + i, hash);
            bytes memory signature = abi.encodePacked(r, s, v);
            multisignatureBytes = bytes.concat(multisignatureBytes, signature);
        }
        weights[0] = 1;

        // use invalid keys
        signers[0] = vm.addr(PRIVATE_KEY_BASE + 1);
        signers[1] = vm.addr(PRIVATE_KEY_BASE);

        (bool valid, MultisignatureChecker.Error err) = MultisignatureChecker.isValidWeightedMultiSignature(
            signers,
            weights,
            10,
            hash,
            multisignatureBytes
        );
        require(valid == false, "valid == false");
        require(err == MultisignatureChecker.Error.InvalidArrayLength, "err == InvalidArrayLength");
    }
}
