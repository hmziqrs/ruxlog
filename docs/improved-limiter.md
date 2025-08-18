# Abuse Limiter v2 (Atomic Redis Lua Script)

This document proposes an improved, race-free abuse limiter for distributed environments using a single atomic Redis Lua script. The goal is correctness under concurrency, predictable sliding-window behavior, and cleaner integration with the application’s error model.

## Goals

- Prevent race conditions and under-counting during bursts.
- Maintain a true sliding window for short and long ranges.
- Avoid panics; provide stable error mapping and observability.
- Return structured results, including remaining block time (retry-after).
- Keep existing call sites working via a thin compatibility wrapper.

## Problems in Current Implementation (`src/services/abuse_limiter.rs`)

- __Panic-prone__: multiple `.unwrap()` calls can crash request paths on transient Redis or parse errors.
- __Undercounting__: ZSET member equals score (unix seconds). Multiple attempts in the same second overwrite the member, undercounting attempts.
- __Race conditions__: Multi-step flow (`GET` → `ZADD` → `EXPIRE` → `ZCOUNT` → `SET`) allows parallel requests to slip through before a block is set.
- __Drift/duplication__: Both absolute `blocked_until` and TTL are maintained; TTL should be the source of truth.
- __TTL misalignment__: Attempts key uses `block_duration` for TTL; it should be tied to the longest counting window.
- __No pruning__: Old entries not pruned from the ZSET; relies only on key TTL.
- __Hardcoded messages__: Messages don’t reflect action/durations dynamically.

## High-Level Design

- Execute limiter logic as a single atomic Lua script on Redis:
  - Use Redis server time (`TIME`) to avoid client clock skew.
  - Prune old attempts beyond the longest window.
  - Add a unique attempt member to a ZSET with score = current time.
  - Count attempts in short and long windows.
  - Conditionally set a block key with TTL (rely on TTL for remaining time).
  - Return a structured result with `allowed`, `retry_after`, counts, and `reason`.
- Expose a Rust API that returns either `Allowed` or `Blocked{retry_after, scope}`, and keep a backward-compatible wrapper returning `Result<(), ErrorResponse>`.

## Redis Keys and Data Model

- `abuse_limiter:attempts:{prefix}`: ZSET of attempts
  - Member: `"<epochSec>:<seq>"` (unique per attempt)
  - Score: `<epochSec>` (i64; convert to f64 at ZADD boundary)
- `abuse_limiter:block:{prefix}`: String key used only for TTL
  - TTL is authoritative unblock time
  - Value content is irrelevant; we won’t parse it
- `abuse_limiter:seq:{prefix}`: Integer sequence for uniqueness

## Threshold Semantics

- Recommended: block on `>=` threshold, i.e., the Nth attempt triggers block.
- If current behavior intentionally allows the Nth and blocks the (N+1)th, use `>` instead. Documented choice must be consistent across windows.

## Lua Script Behavior (Single Atomic Operation)

Inputs (via KEYS and ARGV):
- KEYS[1] = attempts_key
- KEYS[2] = block_key
- KEYS[3] = seq_key
- ARGV[1] = temp_window_secs
- ARGV[2] = temp_threshold
- ARGV[3] = temp_block_duration_secs
- ARGV[4] = long_window_secs
- ARGV[5] = long_threshold
- ARGV[6] = long_block_duration_secs
- ARGV[7] = attempts_ttl_secs  (max(temp_window, long_window) + slack)

Steps:
1. now = redis.call('TIME'); now_sec = tonumber(now[1])
2. Prune: redis.call('ZREMRANGEBYSCORE', attempts_key, '-inf', now_sec - max_window)
3. seq = redis.call('INCR', seq_key)
4. member = string.format('%d:%d', now_sec, seq)
5. redis.call('ZADD', attempts_key, now_sec, member)
6. redis.call('EXPIRE', attempts_key, attempts_ttl_secs)
7. short_count = redis.call('ZCOUNT', attempts_key, now_sec - temp_window_secs, now_sec)
8. long_count  = redis.call('ZCOUNT', attempts_key, now_sec - long_window_secs, now_sec)
9. If short_count >= temp_threshold then
   - NX set block: redis.call('SET', block_key, '1', 'EX', temp_block_duration_secs, 'NX')
   - ttl = redis.call('TTL', block_key); return {0, math.max(ttl, 0), short_count, long_count, 'temp'}
10. Else if long_count >= long_threshold then
   - NX set block: redis.call('SET', block_key, '1', 'EX', long_block_duration_secs, 'NX')
   - ttl = redis.call('TTL', block_key); return {0, math.max(ttl, 0), short_count, long_count, 'long'}
11. Else return {1, 0, short_count, long_count, 'none'}

Return format:
- `{allowed:int, retry_after_secs:int, short_count:int, long_count:int, reason:string}`

Note: If the block key already exists, the NX won’t replace it; `TTL` still yields the correct remaining time.

## Rust API Changes

New types:
- `enum BlockScope { Temp, Long }`
- `enum LimiterDecision { Allowed, Blocked { scope: BlockScope, retry_after_secs: u64, short_count: u64, long_count: u64 } }`

Functions:
- `check(redis_pool, key_prefix, config) -> Result<LimiterDecision, ErrorResponse>`
  - Executes the Lua script and parses the return.
  - Replaces `.unwrap()` with proper error mapping.
  - On Redis connectivity/timeouts, map errors to `ErrorCode::ServiceUnavailable` (or `ErrorCode::InternalServerError` for unexpected parsing issues) with details in debug builds.
- Backward-compatible wrapper (preserve current signature):
  - `limiter(...) -> Result<(), ErrorResponse>`
  - Calls `check(...)`. On `Blocked`, return `ErrorResponse::new(ErrorCode::TooManyAttempts)` (or `ErrorCode::RateLimited` for generic flows)
    - `.with_message("Too many attempts. Try again in X seconds.")`
    - `.with_retry_after(X)` so `Retry-After` header is set automatically and `context.retryAfter = X` is mirrored.

Config alignment:
- Use i64 seconds for times; convert to f64 only when required by Redis ZSET.
- Attempts TTL = `max(temp_window, long_window) + slack` (e.g., +60s).

## ErrorResponse and Headers

- Adopted: add `retry_after: Option<u64>` to `ErrorResponse` and automatically set the `Retry-After` header in `ErrorResponse`'s `IntoResponse` when present.
- Also mirror `retry_after` to `context.retryAfter` for clients that rely on the JSON field.
- Use existing `ErrorCode` values from `src/error/codes.rs`:
  - Prefer `ErrorCode::TooManyAttempts` for auth/verification flows (maps to 429).
  - Prefer `ErrorCode::RateLimited` for generic rate-limits outside auth.

## Controller Integration Example (`src/modules/email_verification_v1/controller.rs`)

- Keep call site intact using the wrapper (`abuse_limiter::limiter(...)`).
- When blocked, the returned `ErrorResponse` includes:
  - `code = ErrorCode::TooManyAttempts`, `status = 429`
  - Message like: "Too many attempts for verification code. Try again in X seconds."
  - `retry_after = Some(X)` so the `Retry-After: X` header is automatically added.
  - `context.retryAfter = X` mirrored for clients reading JSON only.

## Logging and Metrics

- Log at debug level on each decision:
  - `key_prefix`, `short_count`, `long_count`, `decision`, `retry_after_secs`.
- Emit counters if a metrics system is available (e.g., `limiter_blocked_total{scope}`).

## Testing

- Unit tests (logic-level):
  - Threshold semantics (`>=` vs `>`), window edge cases.
- Integration tests (with Redis):
  - Burst tests to ensure atomic blocking.
  - TTL expiry unblocks as expected.
  - Multiple attempts within same second are counted properly (unique members).
  - Idempotence across concurrent calls (no double-block).
- Update smoke tests if any behavior/messages changed.

## Rollout Plan

1. Implement Lua script and `check(...)` API; keep existing `limiter(...)` as wrapper.
2. Canary in `email_verification_v1` only; monitor logs/metrics.
3. Roll out to other call sites.
4. Optional: add feature flag to toggle v2 logic during rollout.

## Alternatives Considered

- MULTI/EXEC (+WATCH): avoids Lua but still adds round-trips and complexity for branching; easier to get wrong under high concurrency.
- Pipelining only: reduces round trips, not atomic; race conditions remain.
- Fixed-window counter (INCR+EXPIRE): simpler but not a true sliding window; coarser control.
- In-memory per-node limiter: very fast but not distributed across instances.

Lua scripting is chosen for correctness and simplicity of atomic sliding-window logic in a distributed system.

## Operational Notes

- Handle Redis errors gracefully: map to `ErrorCode::ServiceUnavailable` (connectivity/timeout) or `ErrorCode::InternalServerError` (unexpected parsing), include details in debug builds.
- Time source: rely on Redis `TIME`, not local clocks.
- Security: key prefixes should avoid user-controlled raw strings; sanitize or prefix with fixed namespaces.
