import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Lottery } from "../target/types/lottery";

import {
  Orao,
  networkStateAccountAddress,
  randomnessAccountAddress,
  FulfillBuilder,
  InitBuilder,
} from "@orao-network/solana-vrf";

import { SystemProgram, Keypair, PublicKey, Transaction, SYSVAR_RENT_PUBKEY, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAccount, createAssociatedTokenAccount, getAssociatedTokenAddress , ASSOCIATED_TOKEN_PROGRAM_ID,createMint, mintTo, mintToChecked, getAccount, getMint, getAssociatedTokenAddressSync,  } from "@solana/spl-token";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";



describe("lottery", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Lottery as Program<Lottery>;
  const provdier = anchor.AnchorProvider.env();
  const connection = program.provider.connection;

  let globalState,lotteryTokenAccount, antcTokenAccount, lotteryAccount: PublicKey;
  let globalStateBump,antcTokenAccountBump, lotteryAccountBump: Number;

  const LOTTERY_STATE_SEED = "Lottery-state-seed";
  const TOKEN_VAULT_SEED = "Token-vault-seed";
  const LOTTERY_START_SEED = "Lottery-start-seed";
  const LOTTERY_TICKET_SEED = "Lottery-ticket-seed";
  const TICKET_SEED = "Ticket-seed";


  const tokenForLottery = new PublicKey("6ag4iXFUbv5NLvrmYVGb4pYcP9DEW6jKASzrSy8HRF8z");
  const tokenForAntc = new PublicKey("FLitGKEPBvBNqPVZbfgRPR5fwcsgSrRv6BDZjxRRFhUC");
  const ownerWalletForLottery = new PublicKey("CSvb7RpkVrdt6p9PhKLe36zDr29SHoGPcrG2F3JDdJUH");
   // Depositer private key -  Don't deposit real money this account :)  
  //  3JU9zBxa3ELaBc7v4HZqXfAimkVtgnBvPeMv4ZrMUEZc - depsoiter
   let depositer = Keypair.fromSecretKey(
    Uint8Array.from([175,124,215,249,71,81,250,130,75,65,94,199,88,84,242,241,140,214,95,88,158,159,51,252,50,83,205,147,147,84,200,23,34,48,243,168,58,75,73,174,20,134,71,70,206,199,213,175,185,104,181,133,171,89,218,189,80,30,65,156,172,127,245,207])
  );

  let owner = Keypair.fromSecretKey(
    Uint8Array.from([40,99,26,70,105,80,7,101,254,157,6,15,246,207,151,29,5,142,33,154,246,128,6,190,239,191,147,115,241,217,13,169,63,7,158,42,242,198,39,230,40,85,41,68,22,57,86,10,229,14,159,81,159,159,3,218,116,30,3,106,54,57,221,134])
  );

  const vrf = new Orao(provdier);

  it("Get PDA", async() => {
    [globalState, globalStateBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_STATE_SEED)
      ],
      program.programId
    );

    lotteryTokenAccount = await getAssociatedTokenAddress(
      tokenForLottery,
      ownerWalletForLottery
    );

    [antcTokenAccount, antcTokenAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(TOKEN_VAULT_SEED),
        tokenForAntc.toBuffer()
      ],
      program.programId
    );
   
  });
  
  it("Is initialized!", async () => {
    // Add your test here.
    let rewardBreakdown = [new anchor.BN(10),new anchor.BN(15),new anchor.BN(20),new anchor.BN(50)];

    try {
      const tx = await program.rpc.initialize(
        rewardBreakdown,
        globalStateBump,
        {
          accounts: {
            globalState,
            tokenForLottery,
            lotteryTokenAccount,
            tokenForAntc,
            antcTokenAccount,
            owner: owner.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID
          },
          signers: [owner]
        }
      );
      console.log("Your transaction signature", tx);
      const globalStateData = await program.account.globalState.fetch(globalState);
      console.log("globalStateData:->", globalStateData);
    } catch (error) {
      console.log(error);
    }
  }); 
  

  
  it("start lottery", async() => {
    const globalStateData = await program.account.globalState.fetch(globalState);
    let lotteryID = 0;

    if (globalStateData == null) {
      lotteryID = 1;
    } else {
       lotteryID = Number(globalStateData.currentLotteryId) + 1;
    }

    [lotteryAccount, lotteryAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_START_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8)
      ],
      program.programId
    );

    const force = Keypair.generate().publicKey;
    ;
  
    const random = randomnessAccountAddress(force.toBuffer());
    const networkState = await vrf.getNetworkState();
    const treasury = networkState.config.treasury;

    try {
      const tx = await program.rpc.startLottery(
        [...force.toBuffer()],
        new anchor.BN(100),
        new anchor.BN(2),
        {
          accounts: {
            globalState,
            lottery: lotteryAccount,
            owner: owner.publicKey,
            random,
            treasury,
            config: networkStateAccountAddress(),
            vrf: vrf.programId,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            clock: SYSVAR_CLOCK_PUBKEY
          },
          signers: [owner]
        }
      );
      console.log("start lottery transaction success: ->", tx);
      const lotteryData = await program.account.lottery.fetch(lotteryAccount);
      console.log("lottery Data: ->", lotteryData);
    } catch (error) {
      console.log(error);
    }
  });  
   
 
  it("buy tickets", async() => {
    const globalStateData = await program.account.globalState.fetch(globalState);
    const lotteryID = Number(globalStateData.currentLotteryId);

    const [lotteryTicket, _1] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_TICKET_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8),
        depositer.publicKey.toBuffer(),
      ],
      program.programId
    );

    let ticketIndex = 0;

    try {
      const lotteryTicketData = await program.account.lotteryTicket.fetch(lotteryTicket);
      ticketIndex= Number(lotteryTicketData.totalTicket) + 1;
    } catch (error) {
      ticketIndex= 1;
      console.log(error);
    }
  
    console.log("ticketIndex->", ticketIndex);

    const [force, _] = await anchor.web3.PublicKey.findProgramAddress(
      [
        new anchor.BN(lotteryID).toBuffer('le',8),
        new anchor.BN(ticketIndex).toBuffer('le',1),
        depositer.publicKey.toBuffer()
      ],
      program.programId
    );
    const PROGRAM_ADDRESS = "VRFzZoJdhFWL8rkvu87LpKM3RbcVezpMEc6X5GVDr7y";
    const PROGRAM_ID = new PublicKey(PROGRAM_ADDRESS);

    const buyerTokenAccount = await getAssociatedTokenAddress(
      tokenForLottery,
      depositer.publicKey
    );

    const adminLotteryTokenAccount = lotteryTokenAccount;

    const random = randomnessAccountAddress(force.toBuffer());
    console.log("random->", random);
    const networkState = await vrf.getNetworkState();
    const treasury = networkState.config.treasury;

    
    [lotteryAccount, lotteryAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_START_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8)
      ],
      program.programId
    );
    let lotteryData = await program.account.lottery.fetch(lotteryAccount);

    const [prevLotteryAccount, prevLotteryAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_START_SEED),
        new anchor.BN(lotteryID-1).toBuffer('le',8)
      ],
      program.programId
    );
    


    const [ticket, ticketBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(TICKET_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8),
        new anchor.BN(ticketIndex).toBuffer('le',1),
        depositer.publicKey.toBuffer()
      ],
      program.programId
    );
  
    try {
      const tx = await program.rpc.buyTickets(
        [...force.toBuffer()],
        new anchor.BN(lotteryID),
        new anchor.BN(ticketIndex),
        {
          accounts: {
            buyer: depositer.publicKey,
            globalState,
            lottery:lotteryAccount,
            lotteryTicket,
            ticket,
            tokenForLottery,
            buyerTokenAccount,
            adminLotteryTokenAccount,
            random,
            treasury,
            config: networkStateAccountAddress(),
            vrf: vrf.programId,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            clock: SYSVAR_CLOCK_PUBKEY
          },
          signers: [depositer]
        }
      );
      // Await fulfilled randomness (default commitment is "finalized"):
      const randomness = await vrf.waitFulfilled(force.toBuffer());
      console.log("Your randomness is " + randomness.fulfilled());
      
    } catch(error) {
      console.log(error);
    }

    try {
      const lotteryRandom = randomnessAccountAddress(lotteryData.force);

      const cliam_ticket_tx = await program.rpc.confirmTickets(
        {
          accounts: {
            user: depositer.publicKey,
            globalState,
            lottery: lotteryAccount,
            lotteryTicket,
            ticket,
            random,
            lotteryRandom,
            systemProgram: SystemProgram.programId,
          },
          signers: [depositer]
        }
      )
    } catch (error) {
      console.log(error)
    }
    lotteryData = await program.account.lottery.fetch(lotteryAccount);
    console.log("lotteryData->",lotteryData);
    const lotteryTicketData = await program.account.lotteryTicket.fetch(lotteryTicket);
    console.log("lotteryTicketData->", lotteryTicketData);
    const ticketData = await program.account.ticket.fetch(ticket);
    console.log("ticketData->", ticketData);
  });
  
  it("calculate the antc amount to deposit and deposit antc", async() => {
    const globalStateData = await program.account.globalState.fetch(globalState);
    let  lotteryID = Number(globalStateData.currentLotteryId);

    [lotteryAccount, lotteryAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_START_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8)
      ],
      program.programId
    );

    [lotteryAccount, lotteryAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_START_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8)
      ],
      program.programId
    );
  

    const lottery_price = 15000;
    const antc_price = 30000;

    const calculate_tx = await program.rpc.calculateAntcForLottery(
      new anchor.BN(lottery_price),
      new anchor.BN(antc_price),
      {
        accounts: {
          owner: owner.publicKey,
          globalState,
          lottery: lotteryAccount,
          systemProgram: SystemProgram.programId
        },
        signers:[owner]
      }
    );
    console.log("cauclate tx ->", calculate_tx);
    const lotteryData = await program.account.lottery.fetch(lotteryAccount);
    console.log("amount_collected_in_lottery_coin ->", Number(lotteryData.amountCollectedInLotteryCoin));
    console.log("deposit amount ->", Number(lotteryData.amountAntcForDeposit));

    const buyerTokenAccount = await getAssociatedTokenAddress(
      tokenForAntc,
      depositer.publicKey
    );

    console.log("antAmount in buyerTokenAccount ->", await getTokenBalanceWeb3(program.provider.connection,buyerTokenAccount));

    try {
      const deposit_tx = await program.rpc.depositAntcForLottery(
        lotteryData.amountAntcForDeposit,
        {
          accounts: {
            owner: depositer.publicKey,
            globalState,
            lottery: lotteryAccount,
            tokenForAntc,
            buyerTokenAccount,
            antcTokenAccount,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
          signers: [depositer]
        }  
      );
  
      console.log("antcAmount in pool ->", await getTokenBalanceWeb3(connection,antcTokenAccount));
    } catch (error) {
      console.log(error);
    }
  }) 
 
  it("close lottery", async() => {
    const globalStateData = await program.account.globalState.fetch(globalState);
    let lotteryID = 0;

    if (globalStateData == null) {
      lotteryID = 1;
    } else {
       lotteryID = Number(globalStateData.currentLotteryId);
    }
    
    [lotteryAccount, lotteryAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_START_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8)
      ],
      program.programId
    );

    const close_tx = await program.rpc.closeLottery(
      new anchor.BN(lotteryID),
      {
        accounts: {
          lottery: lotteryAccount,
          owner: owner.publicKey,
          clock: SYSVAR_CLOCK_PUBKEY
        },
        signers: [owner]
      }
    );
    console.log("close tx->", close_tx);
    const lotteryData = await program.account.lottery.fetch(lotteryAccount);
    console.log("lotteryData->",lotteryData);
  }) 
  
  it("process_draw_final_number_and_make_lottery_claimable_handlerr", async() => {

    let lotteryID = 0;
    const globalStateData = await program.account.globalState.fetch(globalState);
    if (globalStateData == null) {
      lotteryID = 1;
    } else {
       lotteryID = Number(globalStateData.currentLotteryId);
    }
    console.log("current lotteryID->", lotteryID);

    [lotteryAccount, lotteryAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_START_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8)
      ],
      program.programId
    );
    let lotteryData = await program.account.lottery.fetch(lotteryAccount);
    console.log("current lottery Data", lotteryData);

    const [prevLotteryAccount, prevLotteryAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_START_SEED),
        new anchor.BN(lotteryID-1).toBuffer('le',8)
      ],
      program.programId
    );
    // let prevLotteryData = await program.account.lottery.fetch(prevLotteryAccount);


    try{
    
      const tx = await program.rpc.processDrawFinalNumberAndMakeLotteryClaimable(
        new anchor.BN(lotteryID),
        {
          accounts: {
            owner: owner.publicKey,
            lottery: lotteryAccount,
            prevLottery: prevLotteryAccount,
            globalState,
            tokenForAntc,
            antcTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          },
          signers: [owner]
        }
      );
      console.log("tx->", tx);
      lotteryData = await program.account.lottery.fetch(lotteryAccount);
      console.log("lotteryData->",lotteryData);
      console.log("antAmount in antcTokenAccount ->", await getTokenBalanceWeb3(program.provider.connection,antcTokenAccount));

    } catch(error) {
      console.log(error);
    }
   
  })
  
  it("cliam the winner", async() => {
    const globalStateData = await program.account.globalState.fetch(globalState);
    const lotteryID = Number(globalStateData.currentLotteryId);

    [lotteryAccount, lotteryAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_START_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8)
      ],
      program.programId
    );

    const [prevLotteryAccount, prevLotteryAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_START_SEED),
        new anchor.BN(lotteryID-1).toBuffer('le',8)
      ],
      program.programId
    );

    const [lotteryTicket, _1] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(LOTTERY_TICKET_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8),
        depositer.publicKey.toBuffer(),
      ],
      program.programId
    );

    const ticketIndex = 1;
    
    const [ticket, ticketBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(TICKET_SEED),
        new anchor.BN(lotteryID).toBuffer('le',8),
        new anchor.BN(ticketIndex).toBuffer('le',1),
        depositer.publicKey.toBuffer()
      ],
      program.programId
    );

    const buyerTokenAccount = await getAssociatedTokenAddress(
      tokenForAntc,
      depositer.publicKey
    );

    let lotteryData = await program.account.lottery.fetch(lotteryAccount);

    console.log("lotteryData->",lotteryData);
    const lotteryRandom = randomnessAccountAddress(lotteryData.force);
    
    let ticketData = await program.account.ticket.fetch(ticket);
    console.log("ticketData->",ticketData);

    const random = randomnessAccountAddress(ticketData.force);
    
    console.log("antAmount in buyerTokenAccount before ->", await getTokenBalanceWeb3(program.provider.connection,antcTokenAccount));

    try {
      const claim_tx = await program.rpc.claimTickets(
        ticketIndex,
        {
          accounts:{
            user: depositer.publicKey,
            globalState,
            lottery: lotteryAccount,
            prevLottery:prevLotteryAccount,
            lotteryTicket,
            ticket,
            tokenForAntc,
            buyerTokenAccount,
            antcTokenAccount,
            random,
            lotteryRandom,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram:SystemProgram.programId
          },
          signers:[depositer]
        }
      );
      console.log("antAmount in buyerTokenAccount after ->", await getTokenBalanceWeb3(program.provider.connection,antcTokenAccount));
  
    } catch (error) {
      console.log(error);
    }
   
  })
  
});

async function getTokenBalanceWeb3(connection, tokenAccount) {
  const info = await connection.getTokenAccountBalance(tokenAccount);
  if (info.value.uiAmount == null) throw new Error('No balance found');
  console.log(info.value.uiAmount);
  return info.value.uiAmount;
}

