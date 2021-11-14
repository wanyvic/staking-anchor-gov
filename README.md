# 概述
设计一个池子用来收集用户的anc，收完手续费后再次投入anc的gov合约中staking。
# 需求分析
- 用户可以投入、取出、查看自己存入的anc
- 管理员可以设置费率


[![Basic](https://github.com/wanyvic/staking-anchor-gov/actions/workflows/Basic.yml/badge.svg)](https://github.com/wanyvic/staking-anchor-gov/actions/workflows/Basic.yml)
# architecture
## State variables
### Config
| name            | data structure          | option          |
| --------------- | ----------------------- | --------------- |
| `owner`         | `CanonicalAddr`         | 管理员          |
| `pending_owner` | `Option<CanonicalAddr>` | 预备的管理员    |
| `dev`           | `CanonicalAddr`         | 手续费接收地址  |
| `anc_token`     | `CanonicalAddr`         | token地址       |
| `anc_gov`       | `CanonicalAddr`         | gov staking地址 |

| name           | data structure       | option               |
| -------------- | -------------------- | -------------------- |
| `config`       | `Config`             | 合约配置             |
| `fee_rate`     | `Decimal`            | 管理员设置的手续费率 |
| `total_shares` | `Uint128`            | 总计的份额           |
| `user_states`  | `map<addr, Uint128>` | 用户份额的map        |

## functions
### static calls

| func name   | parameter | retuns              | instruction                           |
| ----------- | --------- | ------------------- | ------------------------------------- |
| `Config`    |           | `ConfigResponse`    | 返回`ConfigResponse`                  |
| `UserState` | `String`  | `UserStateResponse` | 根据用户`Addr`返回`UserStateResponse` |
| `State`     |           | `StateResponse`     | 返回`StateResponse`                   |

### dynamic calls

| name                | data structure   | parameter   | option                                                                   |
| ------------------- | ---------------- | ----------- | ------------------------------------------------------------------------ |
| `UpdateDev`         | `String`         | `owner`     | 更新`dev`的地址                                                          |
| `TransferOwnership` | `String`         | `owner`     | 移交`owner`权限                                                          |
| `AcceptedOwner`     |                  | `new owner` | 新的`owner`接受权限                                                      |
| `UpdateFeeRate`     | `Decimal`        | `owner`     | 更新费率                                                                 |
| `Receive`           | `Cw20ReceiveMsg` | `token`     | 处理anc的`Cw20ReceiveMsg`消息。存入token。                               |
| `WithdrawToken`     | `Uint128`        | `user`      | 用户取出anc，如果通过amount计算出的share值大于用户的值，则强制取出最大值 |

## unit testing cases
### static calls
| function testing name | option                            |
| --------------------- | --------------------------------- |
| `query_config`        | 检查`ConfigResponse`数据一致性    |
| `query_user_state`    | 检查`UserStateResponse`数据一致性 |
| `query_state`         | 检查`StateResponse`数据一致性     |
### dynamic calls

| function testing name                                | option                                              |
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
| `proper_receive_with_dev_fee_same_account`           | 检查dev与存款人相同时的存款情况                     |
| `proper_receive_with_dev_fee`                        | 检查存在devfee的存款情况                            |
| `proper_receive_without_dev_fee_double`              | 检查存在feerate为0时的双次存款情况                  |
| `fails_withdraw_token_out_of_amount`                 | 拒绝超过自身上限取款，报`InsufficientFunds`         |
| `fails_withdraw_token_without_deposit`               | 拒绝没有存款的取款，报`NothingStaked`               |
| `proper_withdraw_token`                              | 检查取款后`user_state`的情况                        |
| `proper_withdraw_token_all`                          | 检查取出所有token后`user_state`的情况               |
| `fails_reply_temp_send_data_not_found`               | 成功从gov取回时当`tempSendData`不存在报错           |
| `proper_reply`                                       | 成功从gov取回，再打给用户                           |


## optimizer
```bash
# .zshrc or .bashrc

# set this to whichever latest version of the optimizer is
OPTIMIZER_VERSION="0.11.4"

alias rust-optimizer='docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:${OPTIMIZER_VERSION}'
```
```bash
# in your project folder
rust-optimizer       # if your project contains only 1 contract
```

## unit tests
```
cargo test
```
## integration tests
### 1. install LocalTerra
```bash
git clone --depth 1 https://www.github.com/terra-money/LocalTerra
cd LocalTerra
docker-compose up
```
### 2. run integration tests
```bash
# firstly run rust-optimizer to compile optimizely wasm
rust-optimizer

# secondly use typescript to execute integration tests
cd scripts
npm i
npm i -g ts-node
ts-node main.ts
```