# 概述
设计一个池子用来收集用户的anc，收完手续费后再次投入anc的gov合约中。
# 需求分析
- 用户可以投入、取出、查看自己存入的anc
- 管理员可以设置费率

## 状态数据：
### Config:
| 状态变量        | 类型                    | 备注            |
| --------------- | ----------------------- | --------------- |
| `owner`         | `CanonicalAddr`         | 管理员          |
| `pending_owner` | `Option<CanonicalAddr>` | 预备的管理员    |
| `dev`           | `CanonicalAddr`         | 手续费接收地址  |
| `anc_token`     | `CanonicalAddr`         | token地址       |
| `anc_gov`       | `CanonicalAddr`         | gov staking地址 |


| 状态变量       | 类型                 | 备注                 |
| -------------- | -------------------- | -------------------- |
| `config`       | `Config`             | 合约配置             |
| `fee_rate`     | `Decimal`            | 管理员设置的手续费率 |
| `total_shares` | `Uint128`            | 总计的份额           |
| `user_states`  | `map<addr, Uint128>` | 用户份额的map        |

### 函数
## 静态函数

| 函数名      | 参数     | 返回值              | 说明                                  |
| ----------- | -------- | ------------------- | ------------------------------------- |
| `Config`    |          | `ConfigResponse`    | 返回`ConfigResponse`                  |
| `UserState` | `String` | `UserStateResponse` | 根据用户`Addr`返回`UserStateResponse` |
| `State`     |          | `StateResponse`     | 返回`StateResponse`                   |

## 动态调用

| 函数名称            | 参数             | 调用方      | 备注                                                                     |
| ------------------- | ---------------- | ----------- | ------------------------------------------------------------------------ |
| `UpdateDev`         | `String`         | `owner`     | 更新`dev`的地址                                                          |
| `TransferOwnership` | `String`         | `owner`     | 移交`owner`权限                                                          |
| `AcceptedOwner`     |                  | `new owner` | 新的`owner`接受权限                                                      |
| `UpdateFeeRate`     | `Decimal`        | `owner`     | 更新费率                                                                 |
| `Receive`           | `Cw20ReceiveMsg` | `token`     | 处理anc的`Cw20ReceiveMsg`消息。存入token。                               |
| `WithdrawToken`     | `Uint128`        | `user`      | 用户取出anc，如果通过amount计算出的share值大于用户的值，则强制取出最大值 |

## 测试用例
### 静态调用
| 测试函数名         | 备注                              |
| ------------------ | --------------------------------- |
| `query_config`     | 检查`ConfigResponse`数据一致性    |
| `query_user_state` | 检查`UserStateResponse`数据一致性 |
| `query_state`      | 检查`StateResponse`数据一致性     |
### 动态调用

| 测试函数名                                           | 备注                                                |
| ---------------------------------------------------- | --------------------------------------------------- |
| `proper_initialization`                              | 检查初始化赋值是否正确                              |
| `fails_update_dev_with_unauthorized`                 | 检查调用者是否有权限，报`Unauthorized`              |
| `fails_update_dev_without_validated_address`         | 检查输入不合法，报`GenericErr`                      |
| `proper_update_dev`                                  | 检查`config.dev`是否正确                            |
| `fails_transfer_ownership_with_unauthorized`         | 检查调用者是否有权限，报`Unauthorized`              |
| `fails_transfer_ownership_without_validated_address` | 检查输入不为空，但地址不合法。报`GenericErr`        |
| `proper_transfer_ownership`                          | 检查`config.pendding_owner`是否正确                 |
| `proper_transfer_ownership_with_none`                | 检查`config.pendding_owner`是否为`None`             |
| `fails_accepted_owner_with_unauthorized`             | 检查调用者是否有权限，报`Unauthorized`              |
| `proper_accepted_owner`                              | 检查`config.pendding_owner`和`config.owner`是否正确 |
| `fails_update_feerate_with_unauthorized`             | 检查调用者是否有权限，报`Unauthorized`              |
| `fails_update_feerate_out_of_limits`                 | 检查费率范围，报`FeeRateOutOfLimits`                |
| `proper_update_feerate`                              | 检查`feerate`是否正确                               |
| `fails_receive_with_unauthorized`                    | 拒绝非`anc_token`的调用                             |
| `fails_receive_with_zero_amount`                     | 拒绝零转账，报`InsufficientFunds`                   |
| `proper_receive_with_dev_fee`                        | 检查存在devfee的存款情况                            |
| `proper_receive_without_dev_fee_double`              | 检查存在feerate为0时的双次存款情况                  |
| `fails_withdraw_token_out_of_amount`                 | 拒绝超过自身上限取款，报`InsufficientFunds`         |
| `fails_withdraw_token_without_deposit`               | 拒绝没有存款的取款，报`NothingStaked`               |
| `proper_withdraw_token`                              | 检查取款后`user_state`的情况                        |
| `proper_withdraw_token_all`                          | 检查取出所有token后`user_state`的情况               |
