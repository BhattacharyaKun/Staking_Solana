import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BackendStakingAnchor } from "../target/types/backend_staking_anchor";
import { createMint, getAssociatedTokenAddress, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { createMockNft, getProgramPdaInfo, getUserInfo, getUserStakeInfo } from "./test-utils";
import { expect } from "chai";

describe("backend_staking_anchor", () => 
{
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.BackendStakingAnchor as Program<BackendStakingAnchor>;

  const wallet = anchor.workspace.BackendStakingAnchor.provider.wallet;
  let mint: anchor.web3.PublicKey
  let userTokenAccount: anchor.web3.PublicKey
  let userStakeInfo: anchor.web3.PublicKey
  let pdaNftAccount: anchor.web3.PublicKey
  let userInfo: anchor.web3.PublicKey

  before(async () => {
    const mockNftResult = await createMockNft(
      program.provider.connection,
      wallet.payer
    )

    mint = mockNftResult.mint
    userTokenAccount = mockNftResult.userTokenAccount

    userInfo = await getUserInfo(program, wallet.publicKey)

    userStakeInfo = await getUserStakeInfo(program, wallet.publicKey, mint)

    const pdaInfoResult = await getProgramPdaInfo(
      mint,
      wallet.publicKey,
      userStakeInfo
    )

    pdaNftAccount = pdaInfoResult.pdaNftAccount
  })

  it("Stake", async () => 
  {
    let originalUserInfoAccount: { isInitialized: boolean; pointBalance: anchor.BN; activeStake: number; };
    try { originalUserInfoAccount = await program.account.userInfo.fetch(userInfo) } 
    catch { originalUserInfoAccount = undefined}

    console.log(originalUserInfoAccount);

    const accounts = {
      userNftAccount: userTokenAccount,
      pdaNftAccount: pdaNftAccount,
      mint: mint,
    }

    await program.methods.stake().accounts(accounts).rpc();

    const userInfoAccount = await program.account.userInfo.fetch(userInfo);
    const userStakeInfoAccount = await program.account.userStakeInfo.fetch(userStakeInfo);

    expect(userStakeInfoAccount.staked).to.equal(true);
    expect(userStakeInfoAccount.mint.toBase58()).to.equal(mint.toBase58());
    expect(userInfoAccount.activeStake).to.equal((originalUserInfoAccount?.activeStake ?? 0) + 1);
  });
});
