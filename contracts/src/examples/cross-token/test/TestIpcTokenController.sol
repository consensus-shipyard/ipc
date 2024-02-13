import {IpcTokenController} from "../IpcTokenController.sol";
import {SubnetID} from "../../../structs/Subnet.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";

contract TestIpcTokenController is IpcTokenController  {
    using SafeERC20 for IERC20;

    address private tokenContractAddress;
    constructor(
        address _gateway,
        address _tokenContractAddress,
        SubnetID memory _destinationSubnet,
        address _destinationContract
    ) IpcTokenController(_gateway, _tokenContractAddress, _destinationSubnet, _destinationContract) {
            tokenContractAddress = _tokenContractAddress;
        }

    /* This function is used for testing and should not be deployed to production */
    function receiveAndUnlockOnlyOwner(address recipient, uint256 value) external onlyOwner {

        // Transfer the specified amount of tokens to the receiver
        IERC20(tokenContractAddress).safeTransfer(recipient, value);
    }

}

