# Fund deposit and withdrawal

### Fund

The flow of fund operation i.e crediting funds to the specified account in the specified subnet depends on the supply source used.

In case of `Native` supply

- Payable`fund`  function is called on the Gateway contract on the subnet’s parent
- The amount of`msg.value` is automatically received and stored on the Gateway

In case of `ERC20` supply

- `fundWithToken` function is called with `uint256 amount` argument on the Gateway contract
- The function tries to locks a specified amount into custody. The operation reverts if the effective transferred amount is zero or the total balance in custody hasn’t increased. In case of an inflationary `ERC20` token, the locked amount may be higher than the `amount` initially provided specified. The *actually* locked amount is passed to the top-down message.

In both cases, the top-down message is created and committed for execution on the subnet to credit the funds.

### Withdrawal

In order to withdraw funds from the subnet

- `release` operation is called on Subnet’s Gateway
- bottom-up release message is committed to be executed on the parent (by relayer)
- withdrew funds are burnt by sending to burn address (99)

The local supply of a subnet is always the native coin, so this method doesn't have to deal with ERC20 tokens.

⚠️ There is currently no emergency withdrawal mechanism, which could be used if the subnet validators are malicious.
