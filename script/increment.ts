import {
    Connection,
    Keypair,
    PublicKey,
    SystemProgram,
    Transaction,
    TransactionInstruction,
    clusterApiUrl,
    sendAndConfirmTransaction,
} from '@solana/web3.js';

import * as dotenv from 'dotenv';

dotenv.config();

// Opcodes for the program
const INSTRUCTION_INITIALIZE = 0;
const INSTRUCTION_INCREMENT = 1;

// Helper function to create a Solana connection
async function createConnection(cluster: string): Promise<Connection> {
    // create a wss connection
    const apiUrl = clusterApiUrl('devnet');
    const connection = new Connection(apiUrl, 'finalized');
    return connection;
}

// Helper function to generate a new account
function generateAccount(): Keypair {
    return Keypair.generate();
}

// Function to create the "initialize" instruction
function createInitializeInstruction(
    programId: PublicKey,
    payer: PublicKey,
    accountToInitialize: PublicKey
): TransactionInstruction {
    const keys = [
        { pubkey: accountToInitialize, isSigner: false, isWritable: true },
        { pubkey: payer, isSigner: true, isWritable: false },
    ];

    const initialValue = 0;
    // 1 bytes for the instruction, 8 bytes for the initial value
    const instructionData = Buffer.alloc(9);
    instructionData.writeUInt8(INSTRUCTION_INITIALIZE, 0);
    instructionData.writeUIntLE(initialValue, 1, 6);
    console.log('9 byte buffer, Initial value:', initialValue);

    return new TransactionInstruction({
        keys,
        programId,
        data: instructionData,
    });
}

// Function to create the "increment" instruction
function createIncrementInstruction(programId: PublicKey, accountToIncrement: PublicKey): TransactionInstruction {
    const keys = [{ pubkey: accountToIncrement, isSigner: false, isWritable: true }];

    const instructionData = Buffer.alloc(9);
    instructionData.writeUInt8(INSTRUCTION_INCREMENT, 0);

    return new TransactionInstruction({
        keys,
        programId,
        data: instructionData,
    });
}

async function main() {
    const payerWallet = process.env.PAYER_WALLET;
    const network = process.env.NETWORK || 'devnet';
    const programIdEnv = process.env.PROGRAM_ID;
    const accountId = process.env.ACCOUNT_ID;

    if (!payerWallet || !network || !programIdEnv) {
        console.error('Missing environment variables: see .env file');
        return;
    }

    const programId = new PublicKey(programIdEnv);

    const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(payerWallet)));

    // Create a connection
    const connection = await createConnection(network);

    let account: PublicKey;
    if (!accountId) {
        // Generate accounts

        const accountKey = generateAccount(); // Account to initialize
        console.log('Generated new account with public key:', accountKey.publicKey.toBase58());
        // Calculate rent-exempt minimum balance for the account
        const rentExemption = await connection.getMinimumBalanceForRentExemption(8);

        // Create account with SystemProgram
        const createAccountTransaction = new Transaction().add(
            SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: accountKey.publicKey,
                lamports: rentExemption,
                space: 8,
                programId: programId,
            })
        );

        console.log('Creating account...');
        const tx = await sendAndConfirmTransaction(connection, createAccountTransaction, [payer, accountKey], {
            skipPreflight: true,
        });
        console.log('Transaction:', tx);
        console.log('Account created successfully.');
        account = accountKey.publicKey;

        // Send the initialize instruction
        const initializeInstruction = createInitializeInstruction(programId, payer.publicKey, account);
        const initializeTransaction = new Transaction().add(initializeInstruction);

        console.log('Initializing account...');
        const tx2 = await sendAndConfirmTransaction(connection, initializeTransaction, [payer], { skipPreflight: true });
        console.log('Transaction:', tx2);
        console.log('Account initialized successfully.');
    } else {
        account = new PublicKey(accountId);
    }

    // Send the increment instruction
    const incrementInstruction = createIncrementInstruction(programId, account);
    const incrementTransaction = new Transaction().add(incrementInstruction);
    console.log('Incrementing counter...');
    const tx3 = await sendAndConfirmTransaction(connection, incrementTransaction, [payer], { skipPreflight: true });
    console.log('Transaction:', tx3);
    console.log('Counter incremented successfully.');

    // sleep a bit to ensure finality
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Fetch and print the account data
    const accountInfo = await connection.getAccountInfo(account);
    if (accountInfo) {
        const counterValue = accountInfo.data.readUIntLE(0, 6);
        console.log('Current counter value:', counterValue);
    } else {
        console.error('Failed to fetch account info.');
    }
}

main().catch((err) => {
    console.error(err);
});
