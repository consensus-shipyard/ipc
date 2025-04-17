// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "elliptic-curve-solidity/contracts/EllipticCurve.sol";
import {IPCAddress, Asset} from "../../contracts/structs/Subnet.sol";
import {CallMsg, IpcMsgKind, IpcEnvelope, ResultMsg} from "../../contracts/structs/CrossNet.sol";
import {IIpcHandler} from "../../sdk/interfaces/IIpcHandler.sol";
import {METHOD_SEND, EMPTY_BYTES} from "../../contracts/constants/Constants.sol";

library TestUtils {
    uint256 public constant GX = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;
    uint256 public constant GY = 0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8;
    uint256 public constant AA = 0;
    uint256 public constant BB = 7;
    uint256 public constant PP = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;

    function derivePubKey(uint256 privKey) external pure returns (uint256, uint256) {
        return EllipticCurve.ecMul(privKey, GX, GY, AA, PP);
    }

    function derivePubKeyBytes(uint256 privKey) public pure returns (bytes memory) {
        (uint256 pubKeyX, uint256 pubKeyY) = EllipticCurve.ecMul(privKey, GX, GY, AA, PP);
        return abi.encode(pubKeyX, pubKeyY);
    }

    function deriveValidatorPubKeyBytes(uint256 privKey) public pure returns (bytes memory) {
        (uint256 pubKeyX, uint256 pubKeyY) = EllipticCurve.ecMul(privKey, GX, GY, AA, PP);

        // https://github.com/ethereum/eth-keys/blob/master/README.md#keyapipublickeypublic_key_bytes

        return abi.encodePacked(uint8(0x4), pubKeyX, pubKeyY);
    }

    function getFourValidators(
        Vm vm
    ) internal returns (uint256[] memory validatorKeys, address[] memory addresses, uint256[] memory weights) {
        validatorKeys = new uint256[](4);
        for (uint i = 0; i < 4; i++) {
            validatorKeys[i] = getPrivateKey(i);
        }

        addresses = new address[](4);
        addresses[0] = vm.addr(validatorKeys[0]);
        addresses[1] = vm.addr(validatorKeys[1]);
        addresses[2] = vm.addr(validatorKeys[2]);
        addresses[3] = vm.addr(validatorKeys[3]);

        weights = new uint256[](4);
        vm.deal(vm.addr(validatorKeys[0]), 1);
        vm.deal(vm.addr(validatorKeys[1]), 1);
        vm.deal(vm.addr(validatorKeys[2]), 1);
        vm.deal(vm.addr(validatorKeys[3]), 1);

        weights = new uint256[](4);
        weights[0] = 100;
        weights[1] = 100;
        weights[2] = 100;
        weights[3] = 100;
    }

    function getThreeValidators(
        Vm vm
    ) internal returns (uint256[] memory validatorKeys, address[] memory addresses, uint256[] memory weights) {
        validatorKeys = new uint256[](3);
        for (uint i = 0; i < 3; i++) {
            validatorKeys[i] = getPrivateKey(i);
        }

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

    /// The dummy private keys generated offchain so that the addresses of the private keys
    /// are sorted in ascending order.
    function getPrivateKey(uint256 idx) internal pure returns (uint256 privateKey) {
        if (idx == 0) {
            return 0xaf2be542934f2283d6cb21ee8c80495ab76d63b1071b75276a4a5e7673e4dae6;
        }
        if (idx == 1) {
            return 0xb423cb058c1bd6b4494364de8bcbfd89d534ba709dd6de06dc7e7884dafca7b3;
        }
        if (idx == 2) {
            return 0xc0df835826416ebdfe6372000466c8c5fe1013f8d40448281ab572d11f4aee50;
        }
        if (idx == 3) {
            return 0xf731c1d1a68c1d060817b50dc3d56d2cdfb35a1636ad99ec9c3352e617bb907b;
        }
        if (idx == 4) {
            return 0x3f339e466f1946264212212866a929837cd565423bf1d4b3818870021f2ec12e;
        }
        if (idx == 5) {
            return 0x1ec4b12edb4056925ccc5687ca77bdd002de7af60de2a4be27f318b0a73df2fa;
        }
        if (idx == 6) {
            return 0x072e51399be0cb21e3c50b076d8af6bb35f019a8d3d629b06e0d0c9e440921df;
        }
        if (idx == 7) {
            return 0x8127a61993cac5dfa6f56d440f010e185fe7d308ddfd012a8966c7f353d123de;
        }
        if (idx == 8) {
            return 0x19d47f8e12c86e270438ed1caa284e2f1ffdd0a99420b2b220ea70647799a60f;
        }
        if (idx == 9) {
            return 0x98374b780974d9c11465be371871b6354fb537f2f24a6b985dbd4375b3f558a8;
        }
        revert("more than 10 validators not supported");
    }

    function newValidator(
        uint256 idx
    ) internal pure returns (address addr, uint256 privKey, bytes memory validatorKey) {
        privKey = getPrivateKey(idx);
        bytes memory pubkey = derivePubKeyBytes(privKey);
        validatorKey = deriveValidatorPubKeyBytes(privKey);
        addr = address(uint160(uint256(keccak256(pubkey))));
    }

    /// Generate a list of validators whose addresses are arranged in ascending order.
    function newValidators(
        uint256 n
    ) internal pure returns (address[] memory validators, uint256[] memory privKeys, bytes[] memory validatorKeys) {
        validatorKeys = new bytes[](n);
        validators = new address[](n);
        privKeys = new uint256[](n);

        for (uint i = 0; i < n; i++) {
            (address addr, uint256 key, bytes memory validatorKey) = newValidator(i);
            validators[i] = addr;
            validatorKeys[i] = validatorKey;
            privKeys[i] = key;
        }

        return (validators, privKeys, validatorKeys);
    }

    // function derivePubKey(uint8 seq) internal pure returns (address addr, bytes memory data) {
    //     data = new bytes(65);
    //     data[1] = bytes1(seq);

    //     // use data[1:] for the hash
    //     bytes memory dataSubset = new bytes(data.length - 1);
    //     for (uint i = 1; i < data.length; i++) {
    //         dataSubset[i - 1] = data[i];
    //     }

    //     addr = address(uint160(uint256(keccak256(dataSubset))));
    // }

    function ensureBytesEqual(bytes memory _a, bytes memory _b) internal pure {
        require(_a.length == _b.length, "bytes len not equal");
        require(keccak256(_a) == keccak256(_b), "bytes not equal");
    }

    // Helper function to validate bytes4[] arrays
    function validateBytes4Array(
        bytes4[] memory array1,
        bytes4[] memory array2,
        string memory errorMessage
    ) internal pure {
        require(array1.length == array2.length, errorMessage);
        for (uint i = 0; i < array1.length; i++) {
            require(array1[i] == array2[i], errorMessage);
        }
    }

    function newXnetCallMsg(
        IPCAddress memory from,
        IPCAddress memory to,
        uint256 value,
        uint64 nonce
    ) internal pure returns (IpcEnvelope memory) {
        CallMsg memory message = CallMsg({method: abi.encodePacked(METHOD_SEND), params: EMPTY_BYTES});
        return
            IpcEnvelope({
                kind: IpcMsgKind.Call,
                from: from,
                to: to,
                value: value,
                message: abi.encode(message),
                originalNonce: 0,
                localNonce: nonce
            });
    }
}

contract MockIpcContract is IIpcHandler {
    /* solhint-disable-next-line unused-vars */
    function handleIpcMessage(IpcEnvelope calldata) external payable returns (bytes memory ret) {
        return EMPTY_BYTES;
    }

    function supplySource() public pure returns (Asset memory t) {
        return t;
    }

    function collateralSource() public pure returns (Asset memory t) {
        return t;
    }

    receive() external payable {}
}

contract MockIpcContractFallback is IIpcHandler {
    /* solhint-disable-next-line unused-vars */
    function handleIpcMessage(IpcEnvelope calldata) external payable returns (bytes memory ret) {
        return EMPTY_BYTES;
    }

    fallback() external {
        revert();
    }
}

contract MockIpcContractRevert is IIpcHandler {
    bool public reverted = true;

    /* solhint-disable-next-line unused-vars */
    function handleIpcMessage(IpcEnvelope calldata) external payable returns (bytes memory) {
        // success execution of this methid will set reverted to false, by default it's true
        reverted = false;

        // since this reverts, `reverted` should always be true
        revert();
    }

    fallback() external {
        console.log("here2");
        revert();
    }
}

contract MockIpcContractPayable is IIpcHandler {
    /* solhint-disable-next-line unused-vars */
    function handleIpcMessage(IpcEnvelope calldata) external payable returns (bytes memory ret) {
        return EMPTY_BYTES;
    }

    receive() external payable {}
}

contract MockFallbackContract {
    fallback() external payable {}
}

contract MockIpcContractResult is IIpcHandler {
    ResultMsg _result;
    bool _hasResult;

    function handleIpcMessage(IpcEnvelope calldata envelope) external payable returns (bytes memory) {
        if (envelope.kind == IpcMsgKind.Result) {
            _result = abi.decode(envelope.message, (ResultMsg));
            _hasResult = true;
            return EMPTY_BYTES;
        }

        return EMPTY_BYTES;
    }

    function hasResult() public view returns (bool) {
        return _hasResult;
    }

    function result() public view returns (ResultMsg memory) {
        return _result;
    }
}
