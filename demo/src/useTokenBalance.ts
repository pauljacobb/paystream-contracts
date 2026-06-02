// SPDX-License-Identifier: Apache-2.0
import { useState, useEffect, useCallback } from "react";
import { PayStreamClient } from "@paystream/sdk";
import { CONFIG } from "./config";

const client = new PayStreamClient(CONFIG);

export interface TokenBalanceResult {
  balance: bigint | null;
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

/**
 * Fetches the employer's token balance from the token contract.
 * Returns balance in stroops (raw units).
 */
export function useTokenBalance(
  publicKey: string | null,
  tokenAddress: string | null
): TokenBalanceResult {
  const [balance, setBalance] = useState<bigint | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [tick, setTick] = useState(0);

  const refresh = useCallback(() => setTick((t) => t + 1), []);

  useEffect(() => {
    if (!publicKey || !tokenAddress) {
      setBalance(null);
      return;
    }
    let cancelled = false;
    setLoading(true);
    setError(null);

    client
      .getTokenBalance(tokenAddress, publicKey)
      .then((bal) => {
        if (!cancelled) setBalance(bal);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [publicKey, tokenAddress, tick]);

  return { balance, loading, error, refresh };
}
