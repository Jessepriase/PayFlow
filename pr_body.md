Closes #1005
Closes #1006
Closes #1107
Closes #1108

### Summary

This PR addresses four Stellar Wave issues in a single cohesive change:

### Issue #1005: Schema-version guard on subscription write entrypoints
- Verified `migration::require_current_version()` is called at the top of `subscribe_inner` (the shared entrypoint for both `subscribe` and `subscribe_with_metadata`)
- The existing `test_migration.rs` suite (registered under `#[cfg(test)]`) validates that subscribes are rejected with `SchemaMigrationRequired` when `schema_version < CURRENT_VERSION`
- No behaviour change when schema is already current

### Issue #1006: Global volume cap override enforcement
- **Added** `effective_global_volume_cap(&Env) -> i128` helper shared by enforcement (`check_and_update_global_volume`), reporting (`get_global_volume_cap`), and config (`get_contract_config`)
- **Fixed** `get_contract_config` to return the effective cap (was returning compile-time constant)
- **Updated** `check_and_update_global_volume` to use the helper (was already reading override but now DRY)
- **Added 3 tests**:
  - `test_global_volume_cap_override_lower_enforced` — lowering cap rejects over-cap charges
  - `test_global_volume_cap_override_raise_allows_more` — raising cap permits previously rejected amounts
  - `test_get_contract_config_reports_effective_cap` — config reflects override when set

### Issue #1107: Scripts module layout documentation
- Added "Module Map: Entrypoints, Shared Libraries, and Harnesses" section to `scripts/README.md`
- Classifies all 40+ scripts into three categories with a reference table
- Documents shared helper locations (ScVal, logging, config, RPC client, dry-run stats, forecasting, DB schema)

### Issue #1108: Environment variable reference + .env.example fixes
- **Fixed** `scripts/.env.example`:
  - Canonical `SECRET_KEY` replaces deprecated `KEEPER_SECRET` (with alias comment)
  - Canonical `NETWORK_PASSPHRASE` documents deprecated `NETWORK_PASSTHRASE` typo alias
- **Created** `docs/scripts-environment.md` with:
  - Canonical variables table (18 vars: type, constraints, defaults, used-by, purpose)
  - Deprecated aliases table (with fallback behavior)
  - Stale/unread variables table (historical context)
  - Script-to-variable matrix (30 scripts × variables)
- **Added** parity test in `scripts/config.test.ts` validating docs match `ConfigSchema` keys

### Pre-existing compilation blockers fixed (unrelated but required for CI)
- Removed duplicate `MAX_SUBSCRIPTION_INTERVAL` constant in `lib.rs`
- Removed duplicate `set_token` in `storage.rs`
- Removed duplicate event publishers in `events.rs`
- Removed duplicate `BATCH_SIZE` in `config.ts`
- Fixed extra `}` in `batch.rs` and `whitelist.rs`
- Removed `config.ts` from `.duplicate-lint-baseline.json`

### Verification
| Gate | Result |
|------|--------|
| `cargo check --all-targets` (contract) | ✅ Passes (only pre-existing deprecation warnings) |
| `cargo check --tests` (contract) | ✅ Passes |
| `npm test` (scripts: config.test.ts) | ✅ 9/9 tests pass |
| `npm run lint:duplicates` (scripts) | ✅ Passes |

### Breaking Changes
None. The global volume cap override now *enforces* where it previously only *reported* — this is the intended fix for #1006. The `get_contract_config` return value now reflects the override (was constant).