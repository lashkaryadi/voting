import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { Voting } from "../target/types/voting";

const IDL = require("../target/idl/voting.json");
const PROGRAM_ID = new PublicKey(IDL.address);

describe("Voting", () => {
  let context;
  let provider;
  let votingProgram: anchor.Program<Voting>;

  let voter1 = Keypair.generate(); 

  beforeAll(async () => {
    context = await startAnchor("", [{ name: "voting", programId: PROGRAM_ID }], []);
    provider = new BankrunProvider(context);
    votingProgram = new anchor.Program<Voting>(IDL, provider);
  });

  it("initializes a poll", async () => {
    await votingProgram.methods
      .initializePoll(
        new anchor.BN(1),
        "What is your favorite color?",
        new anchor.BN(100),
        new anchor.BN(1739370789)
      )
      .rpc();

    const [pollAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, "le", 8)],
      votingProgram.programId
    );

    const poll = await votingProgram.account.poll.fetch(pollAddress);

    console.log(poll);

    expect(poll.pollId.toNumber()).toBe(1);
    expect(poll.description).toBe("What is your favorite color?");
    expect(poll.pollStart.toNumber()).toBe(100);
  });

  it("initializes candidates", async () => {
    await votingProgram.methods.initializeCandidate("Pink", new anchor.BN(1)).rpc();
    await votingProgram.methods.initializeCandidate("Blue", new anchor.BN(1)).rpc();

    const [pinkAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, "le", 8), Buffer.from("Pink")],
      votingProgram.programId
    );
    const pinkCandidate = await votingProgram.account.candidate.fetch(pinkAddress);
    console.log(pinkCandidate);
    expect(pinkCandidate.candidateVotes.toNumber()).toBe(0);
    expect(pinkCandidate.candidateName).toBe("Pink");

    const [blueAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, "le", 8), Buffer.from("Blue")],
      votingProgram.programId
    );
    const blueCandidate = await votingProgram.account.candidate.fetch(blueAddress);
    console.log(blueCandidate);
    expect(blueCandidate.candidateVotes.toNumber()).toBe(0);
    expect(blueCandidate.candidateName).toBe("Blue");
  });

  it("allows voting but prevents duplicate votes", async () => {
    await votingProgram.methods
      .vote("Pink", new anchor.BN(1))
      .signers([voter1])
      .rpc();
    
    try {
      await votingProgram.methods
        .vote("Pink", new anchor.BN(1))
        .signers([voter1])
        .rpc();
      throw new Error("Voter should not be able to vote twice!");
    } catch (err) {
      console.log("Duplicate vote attempt detected:", err.message);
      expect(err.message).toContain("You have already voted.");
    }

    const [pinkAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, "le", 8), Buffer.from("Pink")],
      votingProgram.programId
    );
    const pinkCandidate = await votingProgram.account.candidate.fetch(pinkAddress);
    console.log(pinkCandidate);
    expect(pinkCandidate.candidateVotes.toNumber()).toBe(1);
  });

  it("updates candidate count in poll", async () => {
    await votingProgram.methods
      .initializeCandidate("Green", new anchor.BN(1))
      .rpc();

    const [pollAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, "le", 8)],
      votingProgram.programId
    );

    const poll = await votingProgram.account.poll.fetch(pollAddress);
    console.log("Candidate count:", poll.candidateAmount.toNumber());
    expect(poll.candidateAmount.toNumber()).toBe(3); // 2 previous + 1 new = 3
  });
});
