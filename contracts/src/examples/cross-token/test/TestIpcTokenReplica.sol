import {IpcTokenReplica} from "../IpcTokenReplica.sol";
import {SubnetID} from "../../../structs/Subnet.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";

contract TestIpcTokenReplica is IpcTokenReplica  {
    using SafeERC20 for IERC20;

    constructor(
        address _gateway,
        address _controller,
        SubnetID memory _controllerSubnet
    ) IpcTokenReplica (_gateway, _controller, _controllerSubnet){
    }

    /* This function is used for testing and should not be deployed to production */
    function mintOnlyOwner(address recipient, uint256 value) external onlyOwner {
        _mint(recipient, value);
    }
}
