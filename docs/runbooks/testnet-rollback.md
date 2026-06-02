# Testnet Deployment Rollback Procedure

Soroban contracts are immutable once deployed — you cannot overwrite a contract at an existing ID. "Rolling back" means redeploying the previous WASM and pointing consumers to the new contract IDs.

---

## When to Roll Back

- Smoke test fails after a deployment (`stream_count` check or initialisation step errors out)
- A regression is discovered on testnet after a successful deploy
- A bad commit was merged to `main` and the deploy workflow ran

---

## Step-by-Step Rollback

### 1. Identify the last good commit

```bash
git log --oneline main
```

Note the SHA of the last known-good commit (call it `<GOOD_SHA>`).

### 2. Check out that commit locally and build

```bash
git checkout <GOOD_SHA>
stellar contract build
```

### 3. Deploy the previous WASM to testnet

```bash
export STELLAR_SOURCE_ACCOUNT=<YOUR_DEPLOYER_KEY_NAME>
./scripts/deploy-testnet.sh
```

This prints new `TOKEN_CONTRACT_ID` and `STREAM_CONTRACT_ID` values.

### 4. Initialise the redeployed contracts

```bash
export STELLAR_ADMIN_ADDRESS=<ADMIN_PUBLIC_KEY>
export TOKEN_CONTRACT_ID=<FROM_STEP_3>
export STREAM_CONTRACT_ID=<FROM_STEP_3>
./scripts/init-testnet.sh
```

### 5. Update consumers

Any service or demo app reading contract IDs from environment variables or config must be updated to the new IDs from step 3.

### 6. Re-run the smoke test manually

```bash
stellar contract invoke \
  --id "$STREAM_CONTRACT_ID" \
  --source "$STELLAR_SOURCE_ACCOUNT" \
  --network testnet \
  -- stream_count
# Expected output: 0
```

### 7. Fix forward on `main`

Revert or fix the bad commit on `main` and push. The `deploy-testnet.yml` workflow will trigger automatically and produce a fresh deployment.

```bash
git revert <BAD_SHA>
git push origin main
```

---

## Required Secrets (GitHub Environment: `testnet`)

| Secret | Description |
|---|---|
| `TESTNET_DEPLOYER_SECRET` | Stellar secret key (`S…`) for the deployer account |
| `TESTNET_ADMIN_ADDRESS` | Stellar public key (`G…`) set as contract admin |

---

## Notes

- Old contract IDs remain on-chain but are effectively abandoned after a rollback.
- The `concurrency: group: testnet-deploy` setting in the workflow prevents overlapping deployments.
- For mainnet rollback procedures, see [`docs/runbooks/mainnet-deploy.md`](mainnet-deploy.md).
