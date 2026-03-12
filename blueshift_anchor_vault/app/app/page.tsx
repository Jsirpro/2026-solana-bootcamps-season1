"use client";

import React, { useState, useEffect } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { AnchorProvider, Program, BN } from "@coral-xyz/anchor";
import idl from "../idl/vault.json";

const PROGRAM_ID = "6k7rcvivNbhrjkuSg4egeywd3KxR4Z7Z57uQHJ3oQhkw";

export default function VaultPage() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [amount, setAmount] = useState("1000000"); // 1 SOL in lamports
  const [isMounted, setIsMounted] = useState(false);
  const [walletEnv, setWalletEnv] = useState<{
    isSecureContext: boolean;
    hasSolanaProvider: boolean;
    hasPhantom: boolean;
  } | null>(null);

  useEffect(() => {
    setIsMounted(true);
    if (typeof window !== "undefined") {
      const w = window as any;
      const phantom = w?.phantom?.solana;
      const solana = w?.solana;
      setWalletEnv({
        isSecureContext: window.isSecureContext,
        hasSolanaProvider: Boolean(phantom || solana),
        hasPhantom: Boolean(phantom?.isPhantom || solana?.isPhantom),
      });
    }
  }, []);

  if (!isMounted) {
    return null; // Prevent SSR mismatch
  }

  const getProgram = () => {
    if (!wallet.publicKey || !wallet.signTransaction) return null;
    const provider = new AnchorProvider(connection, wallet as any, {
      commitment: "confirmed",
    });
    return new Program(idl as any, provider);
  };

  const deposit = async () => {
    const program = getProgram();
    if (!program || !wallet.publicKey) return;

    try {
      await program.methods
        .deposit(new BN(amount))
        .accounts({
          signer: wallet.publicKey,
        })
        .rpc();
      alert("Deposit successful!");
    } catch (err) {
      console.error(err);
      alert("Deposit failed.");
    }
  };

  const withdraw = async () => {
    const program = getProgram();
    if (!program || !wallet.publicKey) return;

    try {
      await program.methods
        .withdraw()
        .accounts({
          signer: wallet.publicKey,
        })
        .rpc();
      alert("Withdraw successful!");
    } catch (err) {
      console.error(err);
      alert("Withdraw failed.");
    }
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-8 gap-4">
      <h1 className="text-4xl font-bold">Blueshift Vault</h1>
      {walletEnv && (!walletEnv.isSecureContext || !walletEnv.hasSolanaProvider) && (
        <div className="max-w-xl text-sm text-red-600 text-center">
          {!walletEnv.isSecureContext && (
            <div>
              This page is not a secure context (HTTPS or localhost). Phantom
              typically will not inject on insecure origins like
              `http://192.168.x.x`.
            </div>
          )}
          {!walletEnv.hasSolanaProvider && (
            <div>
              No injected wallet provider detected (window.phantom/solana).
              Confirm Phantom is installed and unlocked, or use HTTPS/localhost.
            </div>
          )}
        </div>
      )}
      <WalletMultiButton />
      {wallet.connected && (
        <div className="flex flex-col gap-2 mt-4">
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            className="border p-2 rounded"
          />
          <button
            onClick={deposit}
            className="bg-green-500 text-white p-2 rounded"
          >
            Deposit
          </button>
          <button
            onClick={withdraw}
            className="bg-red-500 text-white p-2 rounded"
          >
            Withdraw
          </button>
        </div>
      )}
    </div>
  );
}
