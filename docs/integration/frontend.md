# Frontend Integration Guide

How to interact with PayStream contracts from a JavaScript/TypeScript frontend using the Stellar SDK.

## Prerequisites

```bash
npm install @stellar/stellar-sdk
```

Tested with `@stellar/stellar-sdk` v12+.

---

## 1. Connect Wallet

Use Freighter (or any SEP-7 compatible wallet) to get the user's public key and sign transactions.

```typescript
import { getPublicKey, isConnected, signTransaction } from "@stellar/freighter-api";

async function connectWallet(): Promise<string> {
  if (!(await isConnected())) {
    throw new Error("Freighter wallet not found. Please install the extension.");
  }
  return getPublicKey();
}
```

---

## 2. Build a Contract Client

```typescript
import {
  Contract,
  Networks,
  TransactionBuilder,
  BASE_FEE,
  rpc,
} from "@stellar/stellar-sdk";

const NETWORK_PASSPHRASE = Networks.TESTNET; // use Networks.PUBLIC for mainnet
const RPC_URL = "https://soroban-testnet.stellar.org";
const STREAM_CONTRACT_ID = "C..."; // your deployed stream contract ID

const server = new rpc.Server(RPC_URL);
const contract = new Contract(STREAM_CONTRACT_ID);
```

---

## 3. Create a Stream

```typescript
import { Address, nativeToScVal, xdr } from "@stellar/stellar-sdk";

async function createStream(
  employerPublicKey: string,
  employeePublicKey: string,
  tokenContractId: string,
  depositStroops: bigint,
  ratePerSecond: bigint,
  stopTime: bigint // 0n = no end
): Promise<string> {
  const account = await server.getAccount(employerPublicKey);

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(
        "create_stream",
        new Address(employerPublicKey).toScVal(),
        new Address(employeePublicKey).toScVal(),
        new Address(tokenContractId).toScVal(),
        nativeToScVal(depositStroops, { type: "i128" }),
        nativeToScVal(ratePerSecond, { type: "i128" }),
        nativeToScVal(stopTime, { type: "u64" })
      )
    )
    .setTimeout(30)
    .build();

  const prepared = await server.prepareTransaction(tx);
  const signed = await signTransaction(prepared.toXDR(), {
    networkPassphrase: NETWORK_PASSPHRASE,
  });

  const result = await server.sendTransaction(
    TransactionBuilder.fromXDR(signed, NETWORK_PASSPHRASE)
  );

  if (result.status === "ERROR") {
    throw new Error(`Transaction failed: ${result.errorResult?.toXDR()}`);
  }

  // Poll for confirmation
  return pollForResult(result.hash);
}

async function pollForResult(hash: string): Promise<string> {
  for (let i = 0; i < 10; i++) {
    await new Promise((r) => setTimeout(r, 2000));
    const response = await server.getTransaction(hash);
    if (response.status === "SUCCESS") return hash;
    if (response.status === "FAILED") throw new Error("Transaction failed");
  }
  throw new Error("Transaction not confirmed after 20s");
}
```

---

## 4. Withdraw Earnings

```typescript
async function withdraw(
  employeePublicKey: string,
  streamId: bigint
): Promise<void> {
  const account = await server.getAccount(employeePublicKey);

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(
        "withdraw",
        new Address(employeePublicKey).toScVal(),
        nativeToScVal(streamId, { type: "u64" })
      )
    )
    .setTimeout(30)
    .build();

  const prepared = await server.prepareTransaction(tx);
  const signed = await signTransaction(prepared.toXDR(), {
    networkPassphrase: NETWORK_PASSPHRASE,
  });

  await server.sendTransaction(
    TransactionBuilder.fromXDR(signed, NETWORK_PASSPHRASE)
  );
}
```

---

## 5. Query Stream State

Read-only calls use `simulateTransaction` — no signing or fees required.

```typescript
import { scValToNative } from "@stellar/stellar-sdk";

async function getStream(streamId: bigint): Promise<Record<string, unknown>> {
  const account = await server.getAccount(
    "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN" // any funded account
  );

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call("get_stream", nativeToScVal(streamId, { type: "u64" }))
    )
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(sim)) {
    throw new Error(`Simulation failed: ${sim.error}`);
  }

  return scValToNative(sim.result!.retval);
}

async function getClaimable(streamId: bigint): Promise<bigint> {
  const account = await server.getAccount(
    "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN"
  );

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call("claimable", nativeToScVal(streamId, { type: "u64" }))
    )
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(sim)) {
    throw new Error(`Simulation failed: ${sim.error}`);
  }

  return scValToNative(sim.result!.retval) as bigint;
}
```

---

## 6. Error Handling

PayStream contracts panic with structured error codes. Catch them from simulation or transaction results:

```typescript
function parseContractError(errorXdr: string): string {
  // Contract panics surface as diagnostic events in the XDR
  // Common codes:
  // E001 — rate_per_second must be greater than zero
  // E002 — deposit must be positive
  // E007 — deposit below minimum
  // E008 — rate_per_second exceeds maximum
  // E010 — employer and employee must differ
  return errorXdr; // parse with xdr.DiagnosticEvent for full detail
}

async function safeCreateStream(
  employer: string,
  employee: string,
  token: string,
  deposit: bigint,
  rate: bigint,
  stopTime: bigint
): Promise<string | null> {
  try {
    return await createStream(employer, employee, token, deposit, rate, stopTime);
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    if (msg.includes("E010")) {
      console.error("Employer and employee must be different addresses.");
    } else if (msg.includes("E007")) {
      console.error("Deposit is below the contract minimum.");
    } else {
      console.error("Stream creation failed:", msg);
    }
    return null;
  }
}
```

---

## 7. Frontend-to-backend integration expectations (if you use the PayStream backend)

If your frontend calls the PayStream backend (instead of directly querying Soroban for every read), align with the backend’s conventions:

- **Network selection**: send `X-Network: testnet|mainnet` so the backend uses the correct RPC + database schema.
- **Correlation IDs**: include `X-Correlation-Id` (or let the backend generate/propagate it) so requests can be traced across logs.
- **Idempotency for retries**: for any payment/stream-creation endpoints that can be retried, include an idempotency key header/body field as required by the endpoint implementation.
- **CORS**: the backend should expose the required CORS headers for the frontend origin.

When in doubt, treat these backend headers as part of your frontend contract and pin them in your integration tests.

---

## 8. CDN/static assets delivery (recommended)

The frontend is a static SPA (for example the `demo/` app built with Vite). Recommended delivery approach:

- Build with hashed filenames (Vite does this by default for production builds)
- Host the build output on a CDN
- Cache policy:
  - `index.html` (HTML entrypoints): short TTL (e.g., 60s) + must-revalidate
  - JS/CSS assets (hashed): long TTL (e.g., 1 year) + `immutable`

This guarantees clients receive the newest JS bundle after an upgrade without waiting for manual cache purges.

---

## Further Reading

- [API Reference](../api-reference.md)
- [SDK Examples](../../examples/) — runnable JS, Python, and Rust examples
- [Stellar SDK docs](https://stellar.github.io/js-stellar-sdk/)
- [Freighter API](https://docs.freighter.app/)

