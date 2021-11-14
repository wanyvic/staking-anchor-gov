import * as path from "path";
import BN from "bn.js";
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LocalTerra, MsgExecuteContract } from "@terra-money/terra.js";
import {
    toEncodedBinary,
    sendTransaction,
    storeCode,
    instantiateContract,
    queryTokenBalance,
} from "./helpers";
import { assert } from "chai";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// Variables
//----------------------------------------------------------------------------------------

const terra = new LocalTerra();
const deployer = terra.wallets.test1;
const dev = terra.wallets.test2;
const user1 = terra.wallets.test3;
const user2 = terra.wallets.test4;

let anchorToken: string;
let govContract: string;
let stakingContract: string;

//----------------------------------------------------------------------------------------
// Setup
//----------------------------------------------------------------------------------------

async function setupTest() {
    // Step 1. Upload Anchor Token code
    process.stdout.write("Uploading Anchor Token code... ");

    const cw20CodeId = await storeCode(
        terra,
        deployer,
        path.resolve(__dirname, "../../artifacts/anc.wasm")
    );

    console.log(chalk.green("Done!"), `${chalk.blue("codeId")} = ${cw20CodeId}`);

    // Step 2. Instantiate Anchor Token contract
    process.stdout.write("Instantiating Anchor Token contract... ");


    const tokenResult = await instantiateContract(terra, deployer, deployer, cw20CodeId, {
        name: "Anchor Token",
        symbol: "ANC",
        decimals: 6,
        initial_balances: [{
            address: deployer.key.accAddress,
            amount: "1000000000",
        }
        ],
        mint: {
            minter: deployer.key.accAddress,
        },
    });

    anchorToken = tokenResult.logs[0].events[0].attributes[3].value;

    console.log(chalk.green("Done!"), `${chalk.blue("contractAddress")} = ${anchorToken}`);

    // Step 3. Upload ANC GOV code
    process.stdout.write("Uploading ANC GOV code... ");

    const codeId = await storeCode(
        terra,
        deployer,
        path.resolve(__dirname, "../../artifacts/anc_gov.wasm")
    );

    console.log(chalk.green("Done!"), `${chalk.blue("codeId")} = ${codeId}`);

    // Step 4. Instantiate ANC GOV contract

    process.stdout.write("Instantiating ANC GOV contract... ");

    const govResult = await instantiateContract(terra, deployer, deployer, codeId, {
        "expiration_period": 13443,
        "owner": deployer.key.accAddress,
        "proposal_deposit": "1000000000",
        "quorum": "0.1",
        "snapshot_period": 13443,
        "threshold": "0.5",
        "timelock_period": 40327,
        "voting_period": 94097
    });

    const event = govResult.logs[0].events.find((event) => {
        return event.type == "instantiate_contract";
    });

    govContract = event?.attributes[3].value as string;

    console.log(
        chalk.green("Done!"),
        `${chalk.blue("govContract")} = ${govContract}`
    );

    // Step 5. Upload Staking code
    process.stdout.write("Uploading Staking code... ");

    const stakingCodeId = await storeCode(
        terra,
        deployer,
        path.resolve(__dirname, "../../artifacts/staking_anchor_gov.wasm")
    );

    console.log(chalk.green("Done!"), `${chalk.blue("stakingCodeId")} = ${stakingCodeId}`);

    // Step 6. Instantiate Staking contract

    process.stdout.write("Instantiating Staking contract... ");

    const stakingResult = await instantiateContract(terra, deployer, deployer, stakingCodeId, {
        feerate: "0.02",
        dev: dev.key.accAddress,
        anchor_token: anchorToken,
        anchor_gov: govContract,
    });

    const event1 = stakingResult.logs[0].events.find((event) => {
        return event.type == "instantiate_contract";
    });

    stakingContract = event1?.attributes[3].value as string;

    console.log(
        chalk.green("Done!"),
        `${chalk.blue("stakingContract")} = ${stakingContract}`
    );

    // Step 7. register contract
    process.stdout.write("Register contract... ");

    await sendTransaction(terra, deployer, [
        new MsgExecuteContract(deployer.key.accAddress, govContract, {
            "register_contracts": {
                "anchor_token": anchorToken,
            }
        }),
    ]);

    console.log(chalk.green("Done!"))


    // assert(toEncodedBinary({
    //     staking_tokens: {},
    // }) == "eyJzdGFraW5nX3Rva2VucyI6e319");


    // await sendTransaction(terra, deployer, [
    //     new MsgExecuteContract(deployer.key.accAddress, anchorToken, {
    //         "send": {
    //             msg: toEncodedBinary({
    //                 staking_tokens: {},
    //             }),
    //             amount: "33333",
    //             contract: stakingContract
    //         }
    //     }),
    // ]);


    process.stdout.write("Fund user1 with ANC... ");
    await sendTransaction(terra, deployer, [
        new MsgExecuteContract(deployer.key.accAddress, anchorToken, {
            mint: {
                recipient: user1.key.accAddress,
                amount: "10000000000",
            },
        }),
    ]);
    console.log(chalk.green("Done!"))

    process.stdout.write("Fund user2 with ANC... ");
    await sendTransaction(terra, deployer, [
        new MsgExecuteContract(deployer.key.accAddress, anchorToken, {
            mint: {
                recipient: user2.key.accAddress,
                amount: "10000000000",
            },
        }),
    ]);
    console.log(chalk.green("Done!"))

}
// {
//     "staker": {
//         "address": "terra19c80u5f5cf57vp07t8et24w8ql4utae5879lz6"
//     }
// }
// {
//     "user_state": {
//         "user": "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v"
//     }
// }
// {
//     "balance": {
//         "address" : "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v"
//     }
// }
//----------------------------------------------------------------------------------------
// Test 1. staking tokens
//
// User 1 staking 33333 ANC
// User 1 should receive 32667 shares, 32667 available_balance
//
// Result
// ---
// govBalance ANC  33333
// devBalance shares  666
// user1Balance ANC  9999966667, shares  32667
//----------------------------------------------------------------------------------------
async function testStakingTokens() {
    process.stdout.write("Should staking anc for user 1... ");

    let user1Balance: string = await queryTokenBalance(terra, user1.key.accAddress, anchorToken);
    expect(user1Balance).to.equal("10000000000");
    let govBalance: string = await queryTokenBalance(terra, govContract, anchorToken);
    expect(govBalance).to.equal("0");
    let stakingBalance: string = await queryTokenBalance(terra, stakingContract, anchorToken);
    expect(stakingBalance).to.equal("0");

    await sendTransaction(terra, user1, [
        new MsgExecuteContract(user1.key.accAddress, anchorToken, {
            "send": {
                msg: toEncodedBinary({
                    staking_tokens: {},
                }),
                amount: "33333",
                contract: stakingContract
            }
        }),
    ]);
    user1Balance = await queryTokenBalance(terra, user1.key.accAddress, anchorToken);
    expect(user1Balance).to.equal("9999966667");
    govBalance = await queryTokenBalance(terra, govContract, anchorToken);
    expect(govBalance).to.equal("33333");
    stakingBalance = await queryTokenBalance(terra, stakingContract, anchorToken);
    expect(stakingBalance).to.equal("0");

    let user1Res = await terra.wasm.contractQuery<{ available_balance: string, locked_balance: string, shares: string }>(stakingContract, {
        user_state: { user: user1.key.accAddress },
    });
    expect(user1Res.shares).to.equal("32667");
    expect(user1Res.available_balance).to.equal("32667");
    expect(user1Res.locked_balance).to.equal("0");


    let devRes = await terra.wasm.contractQuery<{ available_balance: string, locked_balance: string, shares: string }>(stakingContract, {
        user_state: { user: dev.key.accAddress },
    });
    expect(devRes.shares).to.equal("666");
    expect(devRes.available_balance).to.equal("666");
    expect(devRes.locked_balance).to.equal("0");

    console.log(chalk.green("Passed!"));


}
//----------------------------------------------------------------------------------------
// Test 2. staking tokens
//
// User 1 withdraw 222 ANC
// User 1 should reduces 222 shares, 222 available_balance
//
// Result
// ---
// govBalance ANC  33111
// devBalance shares  666
// user1Balance ANC 9999966889, shares  32445
//----------------------------------------------------------------------------------------
async function testWithDrawToken() {
    process.stdout.write("Should withdraw token... ");

    await sendTransaction(terra, user1, [
        new MsgExecuteContract(user1.key.accAddress, stakingContract, {
            withdraw_token: {
                amount: "222",
            }
        }),
    ]);


    let user1Balance = await queryTokenBalance(terra, user1.key.accAddress, anchorToken);
    expect(user1Balance).to.equal("9999966889");
    let govBalance = await queryTokenBalance(terra, govContract, anchorToken);
    expect(govBalance).to.equal("33111");
    let stakingBalance = await queryTokenBalance(terra, stakingContract, anchorToken);
    expect(stakingBalance).to.equal("0");

    let user1Res = await terra.wasm.contractQuery<{ available_balance: string, locked_balance: string, shares: string }>(stakingContract, {
        user_state: { user: user1.key.accAddress },
    });

    expect(user1Res.shares).to.equal("32445");
    expect(user1Res.available_balance).to.equal("32445");
    expect(user1Res.locked_balance).to.equal("0");

    console.log(chalk.green("Passed!"));

}

// //----------------------------------------------------------------------------------------
// // Main
// //----------------------------------------------------------------------------------------

(async () => {
    console.log(chalk.yellow("\nStep 1. Info"));

    console.log(`Use ${chalk.cyan(deployer.key.accAddress)} as deployer`);
    console.log(`Use ${chalk.cyan(user1.key.accAddress)} as user 1`);
    console.log(`Use ${chalk.cyan(user2.key.accAddress)} as user 2`);

    console.log(chalk.yellow("\nStep 2. Setup"));

    await setupTest();

    console.log(chalk.yellow("\nStep 3. Tests"));

    await testStakingTokens();
    await testWithDrawToken();

    console.log("");
})();