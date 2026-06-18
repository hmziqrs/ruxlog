export const meta = {
  name: 'ruxlog-crypto-audit',
  description: 'Deep cryptography-focused audit: 10 dedicated scanners covering keys, hashing, sessions, 2FA, webhooks, OAuth, RNG, TLS, timing attacks → adversarial verification → crypto completeness critic',
  phases: [
    { title: 'Scan', detail: '10 crypto-dimension parallel scans' },
    { title: 'Verify', detail: 'Adversarial verification of all crypto findings' },
    { title: 'Critic', detail: 'Crypto completeness critic for missed attack vectors' },
    { title: 'Synthesize', detail: 'Merge, deduplicate, prioritize crypto findings' },
  ],
}

// ============================================================
// PHASE 1: SCAN — 10 parallel crypto-dimension scanners
// ============================================================
phase('Scan')

const FINDING_SCHEMA = {
  type: 'object',
  properties: {
    findings: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          id: { type: 'string', description: 'Unique ID like CRYP-KM-001' },
          title: { type: 'string' },
          severity: { type: 'string', enum: ['critical', 'high', 'medium', 'low', 'info'] },
          category: { type: 'string', description: 'e.g. Key Management, HMAC, RNG, TLS' },
          file: { type: 'string' },
          description: { type: 'string', description: 'Detailed crypto-specific description' },
          impact: { type: 'string' },
          fix: { type: 'string', description: 'Cryptographically sound fix recommendation' },
        },
        required: ['id', 'title', 'severity', 'category', 'description', 'fix'],
      },
    },
  },
  required: ['findings'],
}

const scanners = [
  {
    label: 'scan:key-management',
    prompt: `Deep cryptographic audit of KEY MANAGEMENT in the Ruxlog codebase at /Users/hmziq/os/ruxlog.

You are a cryptography expert. Read the ACTUAL source code and analyze:

1. backend/api/src/main.rs — find the hex_to_512bit_key function (around line 36-45). Analyze:
   - How is COOKIE_KEY transformed into a usable key?
   - Is SHA-512 used as a KDF? Is this appropriate? (SHA-512 alone is NOT a proper KDF — should use HKDF, Argon2, or PBKDF2)
   - What is the actual entropy of the input? (COOKIE_KEY=302dd40cb75d17b6 = 64 bits)
   - Is the derived key length sufficient for the cipher used?

2. ALL .env files (.env.dev, .env.prod, .env.stage, .env.test, .env.remote, backend/.env.docker, .env.example) — analyze EVERY key:
   - COOKIE_KEY — entropy, length, cross-environment reuse
   - CSRF_KEY — is it cryptographically random or human-readable? ('ultra-instinct-goku')
   - NEW_KEY — what is this used for? entropy?
   - Any other keys/secrets

3. backend/api/src/config.rs — how are keys loaded and validated? Any length validation?

4. backend/api/src/utils/ — search for any key generation, key derivation, key rotation code

5. Search for: key rotation mechanisms (do they exist?), key versioning, key compromise response

For each finding: id (CRYP-KM-xxx), title, severity, category, file:line, detailed crypto description, impact, and a cryptographically-sound fix (reference NIST SP 800-108 for KDFs, OWASP for key management).

Look for: insufficient entropy, improper KDF usage, key reuse across purposes/environments, missing key rotation, keys in version control, human-readable keys, short keys.`,
  },
  {
    label: 'scan:password-hashing',
    prompt: `Deep cryptographic audit of PASSWORD HASHING in the Ruxlog codebase at /Users/hmziq/os/ruxlog.

You are a cryptography expert. Read the ACTUAL source code and analyze:

1. backend/api/src/services/auth.rs — the authentication service. Find:
   - Which hashing algorithm is used? (Argon2, bcrypt, scrypt, PBKDF2?)
   - What are the EXACT parameters? (For Argon2: m_cost, t_cost, p_cost, output_len)
   - Are these parameters OWASP-recommended? (Argon2id m=46176, t=1, p=1 as of 2023)
   - How is password verification done? (constant-time?)
   - Is the hash format PHC-compliant?

2. backend/api/Cargo.toml — find the password hashing crate (password-auth, argon2, bcrypt). Check version.

3. Search for "argon", "bcrypt", "hash", "verify", "password" across the backend src.

4. backend/api/src/modules/auth_v1/controller.rs and validator.rs — password handling on register/login/reset.

5. backend/api/src/modules/forgot_password_v1/ — password reset hashing.

6. Analyze:
   - Is there password hash migration when parameters change? (lazy re-hashing on login)
   - Maximum password length DoS (Argon2 with very long input = CPU exhaustion)
   - Are timing leaks present in the login flow (different paths for "user not found" vs "wrong password")?
   - Is the hash stored in a way that allows offline cracking if DB is dumped?

For each finding: id (CRYP-PW-xxx), title, severity, category, file:line, detailed crypto description, impact, fix (reference OWASP Password Storage Cheat Sheet).

Look for: weak hash parameters, missing migration, timing oracles, password length DoS, non-PHC formats, missing pepper.`,
  },
  {
    label: 'scan:session-cookies',
    prompt: `Deep cryptographic audit of SESSION & COOKIE CRYPTOGRAPHY in the Ruxlog codebase at /Users/hmziq/os/ruxlog.

You are a cryptography expert. Read the ACTUAL source code and analyze:

1. backend/api/src/main.rs — find the session layer configuration (around line 450-470). Analyze:
   - What session backend is used? (tower-sessions, cookies, custom?)
   - Is the session cookie SIGNED, ENCRYPTED, or both? (Private vs Public cookie)
   - What cipher/MAC is used for the cookie?
   - The .with_secure(false) setting — what's the crypto impact?

2. backend/api/crates/rux-auth/src/session/ — read ALL files (mod.rs, extractor.rs, state.rs). Analyze:
   - How are session IDs generated? (CSPRNG? length? entropy?)
   - Is session_auth_hash implemented and verified? (find the "optional - implement if needed" comment)
   - Session token storage format

3. backend/api/src/db/sea_models/user_session/ — session storage model.

4. Search for: "session", "cookie", "tower_sessions", "tower-sessions" across backend.

5. Analyze crypto concerns:
   - Is the session ID predictable? (insufficient entropy)
   - Are sessions signed with an HMAC? Is the HMAC key the same as COOKIE_KEY (key reuse)?
   - Can a session cookie be tampered with undetectably?
   - Is there session fixation protection (regeneration on auth)?
   - Are session tokens stored server-side (DB) or purely stateless (JWT-like)?

For each finding: id (CRYP-SESS-xxx), title, severity, category, file:line, detailed crypto description, impact, fix.

Look for: weak session ID RNG, missing signing, key reuse, session fixation, predictable tokens, missing encryption of sensitive session data.`,
  },
  {
    label: 'scan:2fa-totp',
    prompt: `Deep cryptographic audit of 2FA / TOTP / BACKUP CODES in the Ruxlog codebase at /Users/hmziq/os/ruxlog.

You are a cryptography expert. Read the ACTUAL source code and analyze:

1. backend/api/src/utils/twofa.rs — the ENTIRE file. Analyze:
   - How is the TOTP secret generated? (length in bytes? RNG source — thread_rng, OsRng?)
   - What's the secret format? (base32 per RFC 6238?)
   - TOTP step/period (30s standard?)
   - Is the code generation RFC 6238 compliant?
   - How many backup codes are generated? How? (modulo bias present?)
   - How are backup codes hashed? (Argon2, SHA, plaintext?)
   - Is there a constant-time comparison function? (find constant_time_eq) — is it used everywhere it should be?
   - What's the TOTP window/skew tolerance? (allows codes from adjacent time steps)

2. backend/api/src/modules/auth_v1/controller.rs — twofa_setup, twofa_verify, twofa_disable handlers.

3. backend/api/src/db/sea_models/user/model.rs — two_fa_secret, two_fa_backup_codes storage.

4. Analyze crypto concerns:
   - Is the TOTP secret stored ENCRYPTED at rest or plaintext?
   - Is there a time window replay attack? (same TOTP code used twice within 30s window — must track last used)
   - Backup code brute-force (are attempts rate-limited?)
   - Secret generation modulo bias
   - Is the recovery flow secure? (backup codes shouldn't weaken 2FA)

For each finding: id (CRYP-2FA-xxx), title, severity, category, file:line, detailed crypto description, impact, fix (reference RFC 6238, RFC 4226).

Look for: plaintext secret storage, modulo bias, missing constant-time comparison, TOTP replay (no used-code tracking), weak backup codes, over-generous time window.`,
  },
  {
    label: 'scan:webhook-hmac',
    prompt: `Deep cryptographic audit of WEBHOOK SIGNATURE VERIFICATION (HMAC) in the Ruxlog codebase at /Users/hmziq/os/ruxlog.

You are a cryptography expert. This is CRITICAL for billing security. Read the ACTUAL source code of EVERY billing provider's webhook verification:

1. backend/api/src/services/billing/stripe.rs — verify_webhook function. Analyze:
   - Does it correctly parse Stripe-Signature header format (t=timestamp,v1=hexsignature)?
   - Does it compute HMAC over timestamp+"."+body with the correct algorithm (SHA-256)?
   - Does it verify the timestamp is within tolerance (replay attack prevention)?
   - Is the comparison constant-time?

2. backend/api/src/services/billing/paypal.rs — verify_webhook. PayPal uses certificate-based verification (not HMAC) — is it done at all?

3. backend/api/src/services/billing/lemon_squeezy.rs — verify_webhook (X-Signature header, SHA-256 HMAC).

4. backend/api/src/services/billing/paddle.rs — verify_webhook. CRITICAL: check if verification is skipped when signature is empty.

5. backend/api/src/services/billing/polar.rs — verify_webhook. Does it do ANY verification?

6. backend/api/src/services/billing/razorpay.rs — verify_webhook (SHA-256 HMAC over body|secret).

7. backend/api/src/services/billing/mercado_pago.rs — verify_webhook.

8. backend/api/src/services/billing/revolut.rs — verify_webhook.

9. backend/api/src/services/billing/airwallex.rs — verify_webhook.

10. backend/api/src/services/billing/crypto.rs — verify_webhook. Does it verify anything?

11. backend/api/src/services/billing/provider.rs — the trait definition.
12. backend/api/src/services/billing/router.rs — how verification results are used.

For EACH provider, determine: (a) is signature verification implemented at all? (b) if so, is the algorithm correct? (c) is the comparison constant-time? (d) is replay/timestamp checked?

For each finding: id (CRYP-HMAC-xxx), title, severity, category, file:line, detailed crypto description, impact, fix (reference each provider's official webhook docs).

Look for: zero verification, wrong algorithm, non-constant-time comparison, missing timestamp check, empty-signature bypass, unsigned bytes mismatch.`,
  },
  {
    label: 'scan:oauth-crypto',
    prompt: `Deep cryptographic audit of OAUTH / OIDC CRYPTOGRAPHY in the Ruxlog codebase at /Users/hmziq/os/ruxlog.

You are a cryptography expert. Read the ACTUAL source code and analyze:

1. backend/api/crates/rux-auth/src/oauth/ — ALL files (mod.rs, csrf.rs, google.rs, provider.rs). Analyze:
   - How is the OAuth state/CSRF token generated? (entropy, RNG)
   - How is the state token stored and compared? (Redis? constant-time?)
   - Is PKCE (S256) implemented? (code_verifier / code_challenge)
   - How is the authorization code exchanged? (any verification of code integrity?)

2. backend/api/src/modules/google_auth_v1/controller.rs and service.rs. Analyze:
   - The verify_csrf_token function (line ~214) — is comparison constant-time?
   - How is the OAuth state stored in Redis? (key = token value = information leak)
   - Token exchange — is the id_token JWT signature verified? (Google's public keys)
   - Is the nonce validated?

3. backend/api/src/db/sea_models/user/model.rs — google_id, oauth_provider storage.

4. Search for: "oauth", "jwt", "state", "nonce", "pkce", "code_verifier" across backend.

5. Analyze crypto concerns:
   - State token predictability (session fixation via OAuth)
   - Missing JWT signature verification (accepting unsigned tokens)
   - alg=none attack (JWT without signature)
   - Token replay (same auth code used twice)
   - Open redirect in OAuth callback
   - Is the access_token/id_token stored encrypted?

For each finding: id (CRYP-OA-xxx), title, severity, category, file:line, detailed crypto description, impact, fix (reference OAuth 2.0 Security BCP, OIDC spec).

Look for: weak state RNG, non-constant-time state comparison, missing JWT verification, alg=none, missing PKCE, token replay, open redirect.`,
  },
  {
    label: 'scan:rng-entropy',
    prompt: `Deep cryptographic audit of RANDOM NUMBER GENERATION & ENTROPY in the Ruxlog codebase at /Users/hmziq/os/ruxlog.

You are a cryptography expert. Read the ACTUAL source code and analyze:

1. Search the ENTIRE backend for RNG usage:
   - "rand::" — which RNGs? (thread_rng, OsRng, StdRng, SmallRng, random())
   - "thread_rng()" — is this CSPRNG-backed? (yes, but verify usage)
   - ".gen_range" — MODULO BIAS check! (gen_range is unbiased, but manual modulo like n % range IS biased)
   - "random()" — is it used for security-sensitive values?
   - Any custom PRNG (LCG, xorshift) — these are NOT CSPRNG
   - "uuid::" — UUID generation (v4 = random, v1 = time-based = predictable)

2. backend/api/src/utils/twofa.rs — TOTP secret and backup code generation RNG.
3. backend/api/src/modules/email_verification_v1/ — verification code RNG.
4. backend/api/src/modules/forgot_password_v1/ — reset code RNG.
5. backend/api/src/modules/csrf_v1/ — CSRF token generation (if any real generation exists).
6. backend/api/src/services/billing/ — any random order IDs / idempotency keys?

7. Analyze for each random value:
   - Is it security-sensitive? (tokens, codes, secrets MUST use CSPRNG)
   - Is the RNG source a CSPRNG (OsRng/thread_rng)?
   - For ranges: is modulo bias present? (manual % vs gen_range)
   - Is the entropy sufficient? (128 bits for tokens)
   - Are tokens/codes stored as plaintext or hashed?

8. Search frontend crates for RNG too (frontend/ruxlog-shared, oxstore) — any client-side token generation?

For each finding: id (CRYP-RNG-xxx), title, severity, category, file:line, detailed crypto description (include the entropy calculation), impact, fix.

Look for: non-CSPRNG for secrets, modulo bias, short tokens/codes, predictable UUIDs, time-based tokens, seeded RNG.`,
  },
  {
    label: 'scan:hashing-algorithms',
    prompt: `Deep cryptographic audit of HASHING ALGORITHM USAGE in the Ruxlog codebase at /Users/hmziq/os/ruxlog.

You are a cryptography expert. Read the ACTUAL source code and analyze:

1. Search the ENTIRE codebase (backend + frontend crates) for hash algorithm usage:
   - "md5" — BROKEN for security (collision attacks). Find ALL uses. Is it used for security or just caching/integrity?
   - "sha1" / "sha-1" — deprecated. Find uses.
   - "sha2" / "sha256" / "sha512" — acceptable for most uses.
   - "blake2" / "blake3" — acceptable.
   - "crc32" / "fnv" — NOT cryptographic, find misuse.

2. backend/api/Cargo.toml and frontend Cargo.toml files — list all hash-related dependencies and their versions.

3. For each hash use, determine PURPOSE:
   - Password hashing (must be slow: Argon2/bcrypt)
   - HMAC (must be SHA-256+, key required)
   - File integrity / dedup (SHA-256 ok; MD5 risky for collision-based attacks)
   - Content addressing (must be collision-resistant)

4. backend/api/src/services/image_optimizer.rs — image hashing for dedup. What algorithm? Can a collision cause a wrong image to be served?

5. Search for: "Hash", "hasher", "digest", "DefaultHasher" (StdHashMap default = NOT crypto).

6. Analyze:
   - md5 used for security-sensitive purpose? (collision attack → e.g., file upload dedup bypass, where two different files hash same)
   - DefaultHasher (SipHash) used for anything security-relevant? (it's keyed but not for security)
   - Short hashes (truncated SHA) reducing collision resistance?
   - Length extension attacks (SHA-256 without HMAC for auth)?

For each finding: id (CRYP-HASH-xxx), title, severity, category, file:line, detailed crypto description, impact, fix.

Look for: MD5/SHA1 for security, truncated hashes, length-extension vulnerabilities, non-crypto hash for security, missing HMAC.`,
  },
  {
    label: 'scan:data-encryption-rest',
    prompt: `Deep cryptographic audit of DATA ENCRYPTION AT REST & TRANSPORT in the Ruxlog codebase at /Users/hmziq/os/ruxlog.

You are a cryptography expert. Read the ACTUAL source code and analyze:

PART A — DATA AT REST ENCRYPTION:

1. backend/api/src/db/sea_models/user/model.rs — what sensitive fields are stored? (password hash, two_fa_secret, two_fa_backup_codes, google_id, oauth tokens). Which are encrypted vs plaintext?
2. backend/api/src/db/sea_models/payout_account/ — payout/bank account details. Encrypted?
3. backend/api/src/db/sea_models/user_session/ — session tokens. Hashed or plaintext?
4. backend/api/src/modules/billing_v1/ — any API keys / secrets stored?
5. Search for: any "encrypt", "decrypt", "cipher", "aes", "chacha" usage in data storage paths.
6. Search for: "zeroize" crate usage (do secrets get wiped from memory)?

PART B — TRANSPORT SECURITY (TLS):

7. backend/api/src/db/sea_connect.rs — does the DB connection use TLS? (sslmode, require_tls)
8. backend/api/src/services/redis.rs — does Redis connection use TLS?
9. backend/api/src/services/mail/smtp.rs — STARTTLS vs implicit TLS (port 465)?
10. backend/traefik/traefik.prod.yml — TLS version minimum, cipher suites, HSTS.
11. backend/api/src/services/billing/*.rs — do outbound HTTPS calls verify TLS certificates? (danger_accept_invalid_certs?)
12. Search for: "danger_accept_invalid", "accept_invalid", "verify_mode", "tls" in HTTP clients.

For each finding: id (CRYP-ENC-xxx), title, severity, category, file:line, detailed crypto description, impact, fix.

Look for: plaintext sensitive data at rest, missing TLS on DB/Redis, STARTTLS (downgradeable), disabled cert verification, missing HSTS, no field-level encryption.`,
  },
  {
    label: 'scan:timing-side-channels',
    prompt: `Deep cryptographic audit of TIMING & SIDE-CHANNEL ATTACKS in the Ruxlog codebase at /Users/hmziq/os/ruxlog.

You are a cryptography expert. Read the ACTUAL source code and analyze ALL constant-time comparison requirements:

1. backend/api/src/utils/twofa.rs — find the constant_time_eq function. Then SEARCH for every place a secret is compared:
   - TOTP code verification — constant-time?
   - Backup code verification — constant-time?
   - Is constant_time_eq actually CALLED or just defined?

2. backend/api/src/modules/google_auth_v1/controller.rs — verify_csrf_token (line ~214): is the comparison stored == token (TIMING VULNERABLE) or constant-time?

3. backend/api/src/modules/auth_v1/controller.rs — password verification. Does it take the same time for "user not found" vs "wrong password"? (early return = timing oracle enabling user enumeration)

4. backend/api/src/modules/email_verification_v1/controller.rs — verification code comparison.
5. backend/api/src/modules/forgot_password_v1/controller.rs — reset code comparison.

6. backend/api/src/services/billing/*.rs — webhook HMAC comparison. Is it constant-time? (HMAC comparison MUST be constant-time or it leaks the signature byte-by-byte)

7. Search for: "==" comparisons on secrets, tokens, codes, hashes. Every a == b comparison where a or b is secret is a timing vulnerability.

8. Analyze additional side channels:
   - Error message oracles (does "invalid signature" vs "expired timestamp" reveal which check failed?)
   - Different response codes for different failure modes
   - Database query timing (does a hit take longer than a miss?)

For each finding: id (CRYP-SC-xxx), title, severity, category, file:line, detailed crypto description, impact, fix (reference CWE-208 Observable Timing Discrepancy, CWE-204 Observable Response Discrepancy).

Look for: == on secrets, early returns on user-not-found, distinguishing error messages, non-constant-time HMAC comparison, missing constant_time_eq usage.`,
  },
]

const scanResults = await parallel(scanners.map(s => () =>
  agent(s.prompt, { label: s.label, phase: 'Scan', schema: FINDING_SCHEMA })
))

const allFindings = scanResults.filter(Boolean).flatMap(r => r.findings)
log(`Phase 1 complete: ${allFindings.length} crypto findings from ${scanResults.filter(Boolean).length} scanners`)

// Deduplicate by ID
const seen = new Set()
const unique = []
for (const f of allFindings) {
  if (!seen.has(f.id)) { seen.add(f.id); unique.push(f) }
}
log(`After dedup: ${unique.length} unique crypto findings`)

// ============================================================
// PHASE 2: ADVERSARIAL VERIFICATION
// ============================================================
phase('Verify')

const VERDICT_SCHEMA = {
  type: 'object',
  properties: {
    id: { type: 'string' },
    confirmed: { type: 'boolean' },
    adjustedSeverity: { type: 'string', enum: ['critical', 'high', 'medium', 'low', 'info'] },
    evidence: { type: 'string', description: 'Quote the actual vulnerable cryptographic code you found' },
    refuted: { type: 'boolean' },
    refuteReason: { type: 'string' },
    cryptoClassification: { type: 'string', description: 'e.g. CWE-330 Insufficient Entropy, CWE-347 Improper Verification of Signature' },
  },
  required: ['id', 'confirmed', 'adjustedSeverity', 'evidence', 'cryptoClassification'],
}

// Verify ALL findings (crypto audits warrant full verification)
log(`Verifying all ${unique.length} crypto findings adversarially`)

const verifiedResults = await pipeline(
  unique,
  f => agent(
    `You are an adversarial CRYPTOGRAPHY verifier. Verify or refute this crypto finding by reading the ACTUAL source code at /Users/hmziq/os/ruxlog.

CRYPTO FINDING TO VERIFY:
ID: ${f.id}
Title: ${f.title}
Severity: ${f.severity}
Category: ${f.category}
File: ${f.file}
Description: ${f.description}
Fix: ${f.fix}

INSTRUCTIONS:
1. Read the actual file and surrounding cryptographic code
2. Verify the cryptographic claim is TRUE:
   - If it claims weak RNG, confirm the RNG used (e.g., thread_rng vs OsRng, modulo bias)
   - If it claims non-constant-time, confirm the comparison operator
   - If it claims broken HMAC, trace the exact bytes being signed vs verified
   - If it claims insufficient entropy, compute the actual bit count
3. If the finding is WRONG or the code is actually secure, set refuted=true (e.g., if a library handles it securely internally)
4. If confirmed, QUOTE the exact vulnerable line(s) and classify with a CWE
5. Adjust severity based on real-world exploitability
6. Be rigorous — cryptography demands precision. Default to refuted if you cannot SEE the exact flaw.`,
    { label: `verify:${f.id}`, phase: 'Verify', schema: VERDICT_SCHEMA }
  ),
  v => v
)

const verified = verifiedResults.filter(Boolean)
const confirmed = verified.filter(v => v.confirmed)
const refuted = verified.filter(v => v.refuted)
log(`Verified: ${confirmed.length} confirmed, ${refuted.length} refuted, ${verified.length} total`)

// ============================================================
// PHASE 3: CRYPTO COMPLETENESS CRITIC
// ============================================================
phase('Critic')

const CRITIC_SCHEMA = {
  type: 'object',
  properties: {
    gaps: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          id: { type: 'string' },
          title: { type: 'string' },
          severity: { type: 'string' },
          cwe: { type: 'string' },
          description: { type: 'string' },
          fix: { type: 'string' },
        },
        required: ['id', 'title', 'severity', 'description'],
      },
    },
    cryptoAreasNotChecked: {
      type: 'array',
      items: { type: 'string' },
    },
  },
  required: ['gaps', 'cryptoAreasNotChecked'],
}

const criticResult = await agent(
  `You are a cryptography completeness critic. Find crypto attack vectors that all 10 scanners MISSED in /Users/hmziq/os/ruxlog.

EXISTING CRYPTO FINDING IDS (already covered): ${unique.map(f => f.id).join(', ')}

INSTRUCTIONS — investigate these commonly-overlooked crypto concerns:

1. POST-QUANTUM / ALGORITHM AGILITY: Is there any plan for crypto agility (swapping algorithms)?

2. SIDE CHANNELS BEYOND TIMING:
   - Cache timing attacks in crypto operations
   - Error oracle attacks (does decryption error message leak padding? — padding oracle)
   - Search for: error handling around decrypt/verify that distinguishes failure modes

3. KEY/SECRET EXFILTRATION VECTORS:
   - Are crypto keys logged ANYWHERE? (debug!, println!, eprintln!, format!)
   - Are keys passed in URLs? query params? (leaks to logs/proxies)
   - Memory handling — are secrets zeroized? (zeroize crate usage)
   - Search for: secrets in error messages, panic messages

4. DEPENDENCY CRYPTO:
   - Read backend/api/Cargo.toml and Cargo.lock for crypto crate versions (ring, rustls, openssl, sha2, hmac, argon2, aes-gcm, rand, subtle)
   - Any known-vulnerable versions? (check for old rustls < 0.23, old ring, etc.)
   - Any crypto from untrusted sources?

5. PROTOCOL-LEVEL:
   - TLS version minimum (TLS 1.0/1.1 = insecure, must be 1.2+)
   - Cipher suite selection (weak ciphers enabled?)
   - Certificate pinning for billing provider calls?
   - HSTS preload?

6. CRYPTO MISUSE PATTERNS:
   - IV/nonce reuse in any cipher (catastrophic for GCM/CTR)
   - ECB mode (insecure — block patterns)
   - Hardcoded IVs
   - Reusing a key for multiple purposes (e.g., same key for HMAC and encryption)
   - Search for: "Nonce", "Iv", "aes", encryption code

7. ENTROPY SOURCES:
   - Is there a fallback when /dev/urandom unavailable?
   - Any deterministic "random" in tests that leaks to prod?

8. SPECIFIC FILES TO CHECK:
   - backend/api/src/utils/ (all files — telemetry, color, sort)
   - backend/api/src/services/image_optimizer.rs (does it decrypt/verify image signatures?)
   - backend/api/src/extractors/multipart.rs (any crypto on uploads?)
   - backend/backup/ directory (encryption at rest for backups?)
   - backend/docker/ SQL init scripts

For each gap: id (CRYP-GAP-xxx), title, severity, CWE, description, fix.
Also list cryptoAreasNotChecked — what dimensions of crypto weren't scanned.

Be adversarial and thorough. Crypto bugs are subtle.`,
  { label: 'critic:crypto-completeness', phase: 'Critic', schema: CRITIC_SCHEMA }
)

log(`Critic found ${criticResult?.gaps?.length || 0} crypto gaps, ${criticResult?.cryptoAreasNotChecked?.length || 0} unchecked areas`)

// ============================================================
// PHASE 4: SYNTHESIZE
// ============================================================
phase('Synthesize')

const SYNTHESIS_SCHEMA = {
  type: 'object',
  properties: {
    totalCryptoFindings: { type: 'number' },
    confirmed: { type: 'number' },
    refuted: { type: 'number' },
    severityBreakdown: {
      type: 'object',
      properties: {
        critical: { type: 'number' },
        high: { type: 'number' },
        medium: { type: 'number' },
        low: { type: 'number' },
        info: { type: 'number' },
      },
    },
    byCategory: { type: 'object' },
    topCryptoRisks: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          rank: { type: 'number' },
          id: { type: 'string' },
          finding: { type: 'string' },
          cwe: { type: 'string' },
          severity: { type: 'string' },
        },
      },
    },
    overallCryptoPosture: { type: 'string', description: '1-2 paragraph assessment of overall cryptographic security' },
    executiveRecommendation: { type: 'string', description: 'Top 3 actions to fix crypto posture' },
  },
  required: ['totalCryptoFindings', 'confirmed', 'refuted', 'severityBreakdown', 'topCryptoRisks', 'overallCryptoPosture'],
}

// Build category counts from confirmed findings
const categoryCounts = {}
for (const f of unique) {
  const v = verified.find(vr => vr.id === f.id)
  if (!v || v.confirmed) {
    const cat = f.category || 'Unknown'
    categoryCounts[cat] = (categoryCounts[cat] || 0) + 1
  }
}

const synthesis = await agent(
  `Synthesize the complete cryptography audit results.

TOTAL UNIQUE CRYPTO FINDINGS: ${unique.length}
CONFIRMED (adversarially verified): ${confirmed.length}
REFUTED: ${refuted.length}

CONFIRMED FINDINGS (id, title, severity, category, CWE):
${JSON.stringify(confirmed.map(v => {
  const f = unique.find(u => u.id === v.id) || {}
  return { id: v.id, title: f.title, severity: v.adjustedSeverity, category: f.category, cwe: v.cryptoClassification, file: f.file, description: (f.description || '').substring(0, 200) }
}), null, 2)}

REFUTED FINDINGS:
${JSON.stringify(refuted.map(v => ({ id: v.id, reason: (v.refuteReason || '').substring(0, 200) })), null, 2)}

CRITIC GAPS: ${JSON.stringify((criticResult?.gaps || []).map(g => ({ id: g.id, title: g.title, severity: g.severity, cwe: g.cwe })), null, 2)}

CATEGORY COUNTS: ${JSON.stringify(categoryCounts, null, 2)}

Provide:
1. Total crypto findings, confirmed, refuted counts
2. Severity breakdown (using ADJUSTED severities from verification)
3. Findings per category
4. Top 10 crypto risks ranked by severity and exploitability (with CWE classifications)
5. Overall cryptographic posture assessment (1-2 paragraphs)
6. Executive recommendation: top 3 actions`,
  { label: 'synthesize:crypto', phase: 'Synthesize', schema: SYNTHESIS_SCHEMA }
)

log(`Synthesis complete: ${synthesis?.totalCryptoFindings} total, ${synthesis?.confirmed} confirmed`)

return {
  scanFindingsCount: unique.length,
  allFindings: unique,
  verified: verified,
  confirmedCount: confirmed.length,
  refutedCount: refuted.length,
  criticGaps: criticResult?.gaps || [],
  cryptoAreasNotChecked: criticResult?.cryptoAreasNotChecked || [],
  synthesis: synthesis,
}
