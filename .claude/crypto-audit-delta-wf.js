export const meta = {
  name: 'crypto-audit-delta-v2',
  description: 'Deep crypto re-audit of Ruxlog — 18 delta scanners vs prior 88 findings, adversarial verify, 3 critics, targeted resweep, synthesis. Find anything the first pass missed.',
  whenToUse: 'Re-running the cryptography audit to ensure completeness against a prior 88-finding report',
  phases: [
    { title: 'Delta Scan', detail: '18 crypto-dimension scanners, each delta-focused vs prior 88 findings' },
    { title: 'Adversarial Verify', detail: 'every finding verified against real source, CWE-classified' },
    { title: 'Completeness Critics', detail: 'dimension-coverage, delta-vs-prior, missing-areas' },
    { title: 'Targeted Resweep', detail: 're-scan areas the critics flagged as gaps' },
    { title: 'Synthesis', detail: 'dedup, classify NEW vs DUPLICATE/STRENGTHENING, assign CRYP2-* IDs' },
  ],
}

// ---- Schemas ----
const FINDINGS_SCHEMA = {
  type: 'object', additionalProperties: false,
  required: ['dimension', 'findings'],
  properties: {
    dimension: { type: 'string' },
    summary: { type: 'string' },
    findings: {
      type: 'array',
      items: {
        type: 'object', additionalProperties: false,
        required: ['title','severity','cwe','file','line','description','why_crypto','evidence','is_new_vs_prior','confidence'],
        properties: {
          title: { type: 'string' },
          severity: { type: 'string', enum: ['critical','high','medium','low','info'] },
          cwe: { type: 'string' },
          file: { type: 'string' },
          line: { type: 'string' },
          description: { type: 'string' },
          why_crypto: { type: 'string' },
          evidence: { type: 'string' },
          is_new_vs_prior: { type: 'boolean' },
          prior_overlap: { type: 'string' },
          confidence: { type: 'number' },
        },
      },
    },
  },
}

const VERDICT_SCHEMA = {
  type: 'object', additionalProperties: false,
  required: ['confirmed','reasoning','actual_cwe','adjusted_severity','confidence'],
  properties: {
    confirmed: { type: 'boolean' },
    refuted_reason: { type: 'string' },
    reasoning: { type: 'string' },
    actual_cwe: { type: 'string' },
    adjusted_severity: { type: 'string', enum: ['critical','high','medium','low','info'] },
    real_exploitability: { type: 'string' },
    evidence_lines: { type: 'string' },
    is_new_vs_prior: { type: 'boolean' },
    overlap_with_prior: { type: 'string' },
    confidence: { type: 'number' },
  },
}

const CRITIC_SCHEMA = {
  type: 'object', additionalProperties: false,
  required: ['covered_dimensions','missing_dimensions','under_covered','new_gaps','duplicates','completeness_score'],
  properties: {
    covered_dimensions: { type: 'array', items: { type: 'string' } },
    missing_dimensions: { type: 'array', items: { type: 'string' } },
    under_covered: { type: 'array', items: { type: 'string' } },
    new_gaps: {
      type: 'array',
      items: {
        type: 'object', additionalProperties: false,
        required: ['title','severity','cwe','file','why_crypto','dimension'],
        properties: {
          title: { type: 'string' },
          severity: { type: 'string', enum: ['critical','high','medium','low','info'] },
          cwe: { type: 'string' },
          file: { type: 'string' },
          why_crypto: { type: 'string' },
          dimension: { type: 'string' },
        },
      },
    },
    duplicates: { type: 'array', items: { type: 'string' } },
    completeness_score: { type: 'number' },
    summary: { type: 'string' },
  },
}

const SYNTH_SCHEMA = {
  type: 'object', additionalProperties: false,
  required: ['executive_summary','severity_breakdown','top_new_findings','genuinely_missed_areas','completeness_vs_prior','all_findings'],
  properties: {
    executive_summary: { type: 'string' },
    severity_breakdown: {
      type: 'object', additionalProperties: false,
      required: ['critical','high','medium','low','info'],
      properties: { critical:{type:'number'}, high:{type:'number'}, medium:{type:'number'}, low:{type:'number'}, info:{type:'number'} },
    },
    classification: {
      type: 'object', additionalProperties: false,
      required: ['new','strengthening_or_correcting_prior','duplicate_confirming_prior'],
      properties: { new:{type:'number'}, strengthening_or_correcting_prior:{type:'number'}, duplicate_confirming_prior:{type:'number'} },
    },
    top_new_findings: {
      type: 'array',
      items: {
        type: 'object', additionalProperties: false,
        required: ['title','severity','cwe','file','why_new'],
        properties: { title:{type:'string'}, severity:{type:'string'}, cwe:{type:'string'}, file:{type:'string'}, why_new:{type:'string'} },
      },
    },
    genuinely_missed_areas: { type: 'array', items: { type: 'string' } },
    prior_corrections: { type: 'array', items: { type: 'string' } },
    completeness_vs_prior: {
      type: 'object', additionalProperties: false,
      required: ['covered_in_prior','newly_covered','still_missing','completeness_score'],
      properties: { covered_in_prior:{type:'number'}, newly_covered:{type:'number'}, still_missing:{type:'array',items:{type:'string'}}, completeness_score:{type:'number'} },
    },
    all_findings: {
      type: 'array',
      items: {
        type: 'object', additionalProperties: false,
        required: ['id','title','severity','cwe','file','classification','verified'],
        properties: {
          id: { type: 'string' },
          title: { type: 'string' },
          severity: { type: 'string', enum: ['critical','high','medium','low','info'] },
          cwe: { type: 'string' },
          file: { type: 'string' },
          line: { type: 'string' },
          dimension: { type: 'string' },
          description: { type: 'string' },
          classification: { type: 'string', enum: ['new','strengthening_or_correcting_prior','duplicate_confirming_prior'] },
          overlap_ref: { type: 'string' },
          verified: { type: 'boolean' },
        },
      },
    },
  },
}

// ---- Prior findings (so scanners hunt the DELTA, not re-reporting) ----
const PRIOR = [
  'PRIOR CRYPTO AUDIT = 88 findings, 84 confirmed, 3 refuted. DO NOT simply re-report these. Your job is to find what they MISSED, UNDER-covered, or got WRONG. Mark is_new_vs_prior accurately.',
  '',
  'ALREADY-KNOWN prior findings (compact):',
  '- CRYP-KM-001: SHA-512 as KDF for AES-256-GCM cookie key from 64-bit COOKIE_KEY constant 302dd40cb75d17b6 (main.rs hex_to_512bit_key).',
  '- CRYP-KM-002/CRYP-RNG-001: static CSRF secret fallback ultra-instinct-goku (middlewares/static_csrf.rs, modules/csrf_v1).',
  '- CRYP-KM-003 no key rotation/versioning; -004 identical secrets all envs; -005 .env.* committed (.gitignore matches .env not .env.*); -006 getrandom Result swallowed in twofa.',
  '- CRYP-2FA-001 2FA not enforced at login; -002 TOTP secret plaintext at rest; -003 backup codes unsalted SHA-256; -004 no rate limit on TOTP; -005 no used-TOTP tracking (replay); -006 window skew; -007 backup verify not constant-time; -008 REFUTED (codes ARE consumed); -009 TOTP secret in setup response; -010 getrandom ignored.',
  '- CRYP-HMAC-001 Stripe ignores timestamp; -002 PayPal fabricated HMAC; -003 Paddle empty-sig bypass; -004 Polar zero verify; -005 crypto provider zero verify.',
  '- CRYP-SESS-001 Secure=false (main.rs:459); -002 no session regen on login; -003 session_auth_hash dead; -004 cookie key raw SHA-512; -005 user_sessions is audit log; -006 REFUTED (PII not in cookie); -008 sameSite/httpOnly defaults; -009 no server revocation.',
  '- CRYP-ENC-001 TOTP plaintext; -002 backup codes SHA-256; -003 payout/bank JSONB unencrypted; -006 no Postgres TLS; -007 no Redis TLS; -008 STARTTLS; -009 Secure=false; -010 Traefik no TLS/HSTS; -011 reqwest::Client::new no pinning; -012 secrets as plaintext Strings; -013 no field encryption; -005 no zeroize.',
  '- CRYP-OA-001/009 state non-CT ==; -002 state-as-redis-key; -003 no PKCE; -004 id_token JWT not verified; -005 no nonce; -006 email-linking no email_verified; -007/008 redirect no allowlist; -010 state entropy ok; -011 no code replay protection.',
  '- CRYP-RNG-002 6-char codes ~31 bits plaintext; -003 getrandom swallowed; -004 backup ~60 bits; -005 newsletter UUID plaintext; -006 seeded StdRng fake data; -007 CLAIMED no modulo bias (CHECK THIS — backup-code gen uses b[0] % ALPHABET.len()).',
  '- CRYP-HASH-001 raw SHA-512 KDF; -003 SHA-256 backup codes; -004 md5/sha1 declared (NOTE: sha1 IS actually used in utils/twofa.rs:4); -005 media dedup trust hash; -006 custom CT length leak.',
  '- CRYP-SC-001 state non-CT; -002 user-enum timing; -003 polar no verify; -004 CT length leak; -005 TOTP window + backup short-circuit; -006 forgot-pwd enum; -007 verify expired-vs-invalid oracle; -008 webhook no timestamp; -009 error echoes detail.',
  '- CRYP-PW-001 password verify timing; -006 argon2 params not configurable.',
  '- CRYP-GAP-* (20): PayPal wrong algo, Mercado Pago manifest, secrets logged, getrandom ignored, backup codes SHA-256, cookie SHA-512, Secure=false, no TLS pinning, Traefik TLS weak, admin_users.sql password123, reset tokens in query params, seeded RNG reachable, auth user enum, no crypto-agility, no zeroize, image optimizer no hash verify, dual rustls/getrandom, CT length leak, .env.docker in git.',
  '',
  'UNDER-COVERED / LIKELY-MISSED ANGLES TO PURSUE (confirmed by inline scouting):',
  '1. TOTP RFC 6238 algorithm correctness itself (twofa.rs uses Hmac<Sha1>, dynamic truncation) — verify it is correct OR find bugs; prior only audited RNG/enforcement, never the HOTP math.',
  '2. The constant_time_eq_str function is COPY-PASTED into 8 provider files (razorpay, airwallex, revolut, paddle, paypal, lemon_squeezy, mercado_pago, stripe) — are all 8 copies identical? do any diverge? are there ADDITIONAL bugs per copy? twofa.rs has a SEPARATE constant_time_eq (different impl) — compare.',
  '3. jsonwebtoken 10.2.0 is a DIRECT Cargo dependency with ZERO usage anywhere (no use jsonwebtoken in backend or rux-auth) — dead crypto dep; and it means the OAuth id_token is NOT decoded despite a JWT lib being available.',
  '4. backend/backup/email_code/ is ORPHANED duplicate email-verification + forgot-password controller code, NOT routed from backend/api/src — dead/duplicate security code that can silently drift; compare to the live modules/email_verification_v1 + forgot_password_v1.',
  '5. Frontends (frontend/admin-dioxus, consumer-dioxus) use getrandom(wasm_js) + base64 — is any token/secret stored in browser (localStorage/IndexedDB/sessionStorage)? any client-side hashing or embedded secret? frontend/oxcore uses reqwest rustls-tls.',
  '6. Outbound credential crypto: how each provider API key/secret is loaded, cached, and how airwallex/paypal/revolut REFRESH their OAuth access tokens (token caching, refresh-token rotation, secret in URL?).',
  '7. Every individual reqwest::Client::new() call (there are ~30) — timeouts, redirect policy, cert validation, User-Agent leaking. Prior only noted it generically.',
  '8. Email-verification + forgot-password CODE generation: read db/sea_models/email_verification/actions.rs generate_code() and forgot_password/actions.rs — entropy source, storage (hashed or plaintext?), single-use, expiry, timing.',
].join('\n')

const COMMON = [
  'You are a senior cryptography auditor. Read the ACTUAL source files (use Read/Grep/Glob) — never speculate. Quote real line numbers and real code as evidence.',
  'Every finding MUST cite a real file:line and explain WHY it is a crypto defect (not just a code smell). Assign a CWE. Compute entropy in bits where RNG is involved.',
  'This is the user OWN codebase (authorized audit). Be exhaustive. Prefer concrete, verifiable findings over vague concerns.',
].join('\n')

// ---- 18 delta dimensions ----
const DIMENSIONS = [
  { key: 'key-master-secrets', prompt: 'DIMENSION: Key & master-secret lifecycle. Audit backend/api/src/main.rs (env loading, env_with_fallback, hex_to_512bit_key, Key::from vs Key::derive_from), all std::env::var secret reads, state.rs, and whether there is ANY key versioning/rotation/keyring, fail-fast on unset secrets, and whether secrets are logged at startup. Find issues BEYOND the known COOKIE_KEY-SHA512 and identical-envs findings: e.g. is the master key ever compared/logged/passed by &str to functions that log it? Are there OTHER derived keys (HMAC signing key vs AEAD key) that should be domain-separated but are not? Are there keys derived for OTHER purposes (CSRF, JWT, password-reset signing) and are they correctly separated?' },
  { key: 'csrf-tokens', prompt: 'DIMENSION: CSRF protection cryptographically. Audit middlewares/static_csrf.rs, modules/csrf_v1, scripts/generate_csrf.rs, test_utils/csrf.rs (TEST_CSRF_KEY constant — does it equal the production fallback?), and every mutating route. Beyond the known static-secret finding: is the token bound to session/origin/user-agent at all? Is comparison constant-time? Is there any double-submit or synchronizer scheme attempted anywhere? Does the token rotate? Is TEST_CSRF_KEY shipped in a way that could match prod? Are CSRF tokens ever logged?' },
  { key: 'password-hashing', prompt: 'DIMENSION: Password hashing correctness. Audit services/auth.rs (verify_password path, timing — does it hash a dummy on user-not-found to avoid the known timing oracle? Is Argon2 parametrized? Is the PHC string parsed safely? What are the m_cost/t_cost/p_cost and are they adequate?), and wherever passwords are set/changed. Find issues beyond known timing + non-configurable params: e.g. is there a legacy-hash migration path, is the hash compared with == anywhere, is the plaintext password zeroized after use, are passwords ever logged in debug/error, is there a max-length DoS (Argon2 on huge input)?' },
  { key: 'totp-algorithm-correctness', prompt: 'DIMENSION: TOTP/HOTP RFC 6238 ALGORITHM correctness (the prior audit never checked the math). Read utils/twofa.rs fully. Verify: HMAC-SHA1 over 8-byte BE counter (correct?), dynamic truncation offset = hmac[19] & 0x0f (correct for SHA1 20-byte output?), the & 0x7f masking, modulo 10^digits, the window iteration. Is the algorithm RFC-correct? FIND the bugs the prior audit missed: (a) generate_backup_code uses b[0] % ALPHABET.len() — count the alphabet length and confirm modulo bias (this CONTRADICTS prior CRYP-RNG-007 which claimed no modulo bias); (b) verify_totp_code_at loops -window..=window and returns on first constant_time_eq match — does the iteration order / early return leak which counter matched (timing)? (c) generate path (line 55-66) duplicates hmac_truncate_to_digits (200-214) — any divergence bug? (d) SHA1 algorithm-strength concern; (e) does pow10 or u32 truncation cause issues at high digit counts? (f) is the secret length enforced (>=160 bits per RFC)? Quote exact lines.' },
  { key: 'session-cookie-lifecycle', prompt: 'DIMENSION: Session cookie cryptography & lifecycle. Audit main.rs:452-465 (cookie config: Secure/HttpOnly/SameSite/domain/path/max_age, __Host- / __Secure- prefix), session/extractor.rs (session creation, cycle_id on login/privilege change, fixation), and the tower-sessions RedisStore wiring. Beyond known Secure=false + no-regen: is there a cookie-name collision risk, is the session ID itself strong (CSPRNG, length), is there session AuthHash binding (session_auth_hash is dead code per prior — confirm), can a stolen cookie be used after password change, is there absolute vs idle timeout, are cookies set on logout properly expired, does SameSite=Lax suffice for the CSRF scheme used?' },
  { key: 'webhook-per-provider-deep', prompt: 'DIMENSION: Webhook signature verification PER PROVIDER (go deeper than the known cluster). Read EVERY provider verify_webhook() in services/billing/{stripe,paypal,lemon_squeezy,paddle,polar,razorpay,mercado_pago,revolut,airwallex,crypto}.rs and router.rs. For each, verify against the provider REAL spec: correct HMAC algorithm, correct SIGNED STRING (not just body — e.g. Razorpay signs body not headers; Paddle uses k1/k2 with the Paddle public key via Ed25519/RSA NOT HMAC; LemonSqueezy signs raw body with SHA256; Revolut signs with the signing secret over a specific string; Airwallex uses a token + specific header). The prior audit caught the big ones but check the SUBTLE per-spec correctness of the 3 that DO verify (razorpay, revolut, airwallex, lemon_squeezy): correct header name? correct encoding (hex vs base64 vs raw)? Also: is there idempotency/replay dedup beyond HMAC? Is the raw body read before JSON parse (body must be raw bytes for HMAC)?' },
  { key: 'oauth-oidc-deep', prompt: 'DIMENSION: OAuth/OIDC deep. Audit modules/google_auth_v1/controller.rs AND frontend/rux-auth (the OAuth library crate — read its source). Beyond known no-PKCE / id_token-unverified / no-nonce / email_verified / redirect-allowlist / non-CT-state: does the rux-auth lib do anything insecure? Is the authorization code exchanged with PKCE? Is there an alg=none or RS256/HS256 confusion risk if JWT IS ever parsed? Is the access_token leaked in logs/redirects? Is the OAuth state value generated with a CSPRNG? Is there open-redirect via the state/redirect param? Are refresh tokens handled? Does it validate iss/aud/exp on anything?' },
  { key: 'jwt-token-format', prompt: 'DIMENSION: JWT & token-format crypto. jsonwebtoken 10.2.0 is a Cargo dep with ZERO usage. Confirm by grepping. Investigate: is ANY token in the system a JWT (access tokens, API tokens, email tokens, session tokens)? If id_token is not decoded, how IS the OAuth user identified (sub claim extracted how? is the raw id_token trusted without signature)? Are there any hand-rolled base64+HMAC token schemes? Is there an alg confusion surface? Is the dead jsonwebtoken dep a CVE risk (check version vs known advisories)? Also audit any bearer/api-token issuance & validation in the app (state.rs secret_key is S3, not signing — confirm no app-level token signing exists, or find it if it does).' },
  { key: 'email-forgot-token-crypto', prompt: 'DIMENSION: Email-verification & password-reset token crypto (deep — prior only noted 6-char + query-param). Read modules/email_verification_v1/*, modules/forgot_password_v1/*, db/sea_models/email_verification/actions.rs (generate_code), db/sea_models/forgot_password/actions.rs. Determine: (1) the EXACT entropy source of generate_code() — CSPRNG or weak? bits?; (2) is the code/token stored HASHED at rest or PLAINTEXT (read the model/column)?; (3) single-use enforcement?; (4) expiry value & enforcement; (5) is the comparison constant-time or DB-equality (==)?; (6) is the code sent over a channel that could be logged (email, but also any in-app echo)?; (7) does the verify endpoint distinguish expired vs invalid (timing/response oracle)?; (8) rate-limiting entropy (keyed by IP or user_id)? Compare the LIVE modules/ impl with the ORPHANED backend/backup/email_code/ impl for drift.' },
  { key: 'rng-exhaustive', prompt: 'DIMENSION: RNG exhaustiveness. Enumerate EVERY randomness source: utils/twofa.rs (getrandom x2), services/seed/types.rs (seeded_rng, StdRng::seed_from_u64), modules/seed_v1/controller.rs (MANY seed_from_u64 — is the seed module reachable in production builds/routing? CRYP-GAP-013 said reachable — confirm and assess), modules/newsletter_v1 (Uuid::new_v4 token), modules/media_v1 (Uuid path), middlewares/request_id (Uuid), services/billing/crypto.rs (Uuid payment refs), db/sea_models code gen. For each: CSPRNG or not? Result handled? modulo bias? entropy bits? Is ANY security value derived from time/seed/counter? Is rand 0.9 + getrandom 0.2 a version skew bug (rand 0.9 expects getrandom 0.3)? Check Cargo.lock for the actual resolved versions and dual versions.' },
  { key: 'tls-transport-config', prompt: 'DIMENSION: TLS & transport configuration (exhaustive). Audit EVERY reqwest::Client::new() (~30 calls across services/billing/*.rs) — each creates a fresh client with default TLS, no timeout, default redirect policy (follows up to 10), no cert pinning. Find: is danger_accept_invalid_certs ever set? Is .rustls-tls vs native-tls consistent? For db/sea_connect.rs (Postgres) and services/redis.rs: is sslmode/tls configured at all (prior said no — confirm the EXACT connection string / ConnectOptions)? For services/mail (lettre): starttls_relay vs TLS — confirm Tls::Required vs Opportunistic (prior ENC-003 was refuted). For frontend/oxcore reqwest rustls-tls. For traefik/*.yml: min TLS version, cipher suite, HSTS. Is there mTLS anywhere? Is the S3/RustFS connection TLS?' },
  { key: 'secret-hygiene-storage', prompt: 'DIMENSION: Secret hygiene & storage at rest. Audit for zeroize/secrecy ABSENCE (prior noted). Go deeper: (1) which structs hold secrets as plain String and derive Debug (so a tracing::debug / error log of the struct leaks them)? grep #[derive(Debug)] on structs with secret_key/webhook_secret/api_key/access_token/client_secret fields; (2) are secrets ever written to logs/tracing explicitly (search for tracing of provider structs, error!(...err) on reqwest responses that echo bodies/headers)?; (3) field-level encryption: TOTP secret, payout/bank JSONB, recovery codes — all plaintext?; (4) are secrets held in long-lived process state (state.rs) for the whole process lifetime without zeroization?; (5) do Docker/migration/seed files bake secrets (admin_users.sql password123, .env.docker)?' },
  { key: 'outbound-credential-crypto', prompt: 'DIMENSION: Outbound API credential crypto for billing providers. For airwallex.rs, paypal.rs, revolut.rs (these fetch OAuth access tokens), razorpay, mercado_pago, stripe, lemon_squeezy, polar: (1) how are API keys/secrets loaded (env, fallbacks, .unwrap_or_default)?; (2) are access tokens cached in memory, for how long, thread-safely?; (3) is the client_secret ever put in a URL query or logged?; (4) for airwallex, is the signature over the token request correct (it computes a hex of something around line 123 — verify)?; (5) is there idempotency-key crypto (PayPal-Request-Id, etc.) and is it strong?; (6) are sandbox vs live URLs hardcoded (prior v2 audit said 3 providers sandbox — confirm crypto-adjacent: are webhook secrets shared test/prod)?; (7) does any provider trust a response field that should be cryptographically bound (amount, status)?' },
  { key: 'dependency-version-audit', prompt: 'DIMENSION: Dependency version & known-CVE audit (cargo). Read backend/api/Cargo.toml + Cargo.lock. List ALL crypto-relevant crate versions: sha2, sha1 (0.10.6), md5 (0.7.0 — is md5 0.7 even maintained?), hmac, aes-gcm, argon2/password-auth, rustls (0.21.12 AND 0.23.26 — dual), getrandom (0.2 AND 0.3?), rand (0.9.0), jsonwebtoken (10.2.0 — check advisories), ring vs aws-lc-rs, base64, hex, data-encoding, subtle. Flag: (1) any known-CVE version; (2) dual versions of the same crate (rustls, getrandom) and the risk; (3) md5/sha1 as direct deps (sha1 IS used in twofa — is that acceptable for TOTP? md5 truly unused?); (4) is there cargo-audit/cargo-deny in CI?; (5) rustls 0.21 is in maintenance — risk. Cite real versions from Cargo.lock.' },
  { key: 'dead-orphaned-crypto-code', prompt: 'DIMENSION: Dead / orphaned / duplicate security & crypto code. Investigate: (1) backend/backup/email_code/ (orphaned email-verification + forgot-password controllers — grep backend/api/src for any reference; if unreferenced it is dead dup code that can drift from the live modules/ impl — compare them and report drift as a risk); (2) backend/backup/quickwit/ (search engine integration — credentials?); (3) scripts/ (generate_csrf.rs, archive_changes.rs — do they bake/embed secrets or weak randomness?); (4) test_utils/csrf.rs TEST_CSRF_KEY — does it match the prod fallback ultra-instinct-goku?; (5) the jsonwebtoken dead dep; (6) migration/src — any hardcoded secrets/keys in migrations?; (7) any #[cfg(test)] crypto code that could compile into release. Dead crypto code is a future-bug incubator; report each with severity.' },
  { key: 'frontend-wasm-token-crypto', prompt: 'DIMENSION: Frontend (Dioxus WASM) crypto & token handling — UNCOVERED by prior backend-only audit. Audit frontend/admin-dioxus/src and frontend/consumer-dioxus/src: (1) where are auth/session tokens or CSRF tokens stored in the browser — localStorage, sessionStorage, IndexedDB, in-memory, cookie? (localStorage = XSS-stealable); (2) is getrandom(wasm_js) used for anything security-sensitive in the frontend?; (3) any client-side hashing/signing/encryption of payloads?; (4) are API keys or secrets embedded in the WASM binary (grep for hard-coded tokens)?; (5) is the CSRF token (the known static one) fetched and sent correctly, and does the frontend trust it blindly?; (6) any eval/innerHTML of server HTML (stored XSS beating the crypto)?; (7) frontend/oxcore + rux-auth crates — any insecure patterns? Use Glob/Grep on frontend/.' },
  { key: 'constant-time-correctness', prompt: 'DIMENSION: Constant-time & MAC-verification correctness, codebase-wide. (1) The 8 duplicated constant_time_eq_str copies (razorpay, airwallex, revolut, paddle, paypal, lemon_squeezy, mercado_pago, stripe) — READ all 8 and check: are they byte-identical? does each early-return on length mismatch (length oracle)? does any have an additional bug? (2) twofa.rs has a SEPARATE constant_time_eq (line 228) — compare it to the billing one; is it actually constant-time? (3) Are there OTHER secret comparisons using == or != on tokens/codes/hashes/mac states across the codebase (grep "== " near tokens, .eq(, .position on hashes)? (4) Is hmac::Mac::verify_slice (constant-time) ever used, or always manual finalize+compare? (5) The backup-code consume_backup_code uses .position() then constant_time_eq per item — does the iteration count leak how many codes remain? Report every non-constant-time secret comparison with file:line.' },
  { key: 'logging-telemetry-leak', prompt: 'DIMENSION: Secret/token leakage via logs, tracing, errors, telemetry. Grep for: tracing::{info,warn,error,debug,trace} and #[instrument(...)] across backend/api/src where the fields include or could include secrets/tokens/codes/passwords/api_keys/webhook_secrets. Check: (1) #[instrument] with fields = that capture payloads containing codes/tokens (e.g. forgot_password, email_verification, billing); (2) error!(...err) that echoes reqwest response bodies/headers (which contain provider secrets or tokens); (3) Debug-derived structs with secret fields that get logged; (4) the abuse-limiter / metrics that might log keys; (5) request_id / access logs that include query-param tokens (?token=, ?code=). Prior CRYP-GAP-004 only caught S3 secret_key via Debug — find the OTHERS. Report each leak point with file:line and what leaks.' },
]

// ---- Phase 1+2: scan -> adversarial verify (pipeline, no barrier) ----
phase('Delta Scan')
log('Launching 18 delta scanners; each finding is adversarially verified as its scanner completes.')

// Scanners that 429'd on the inherited (opus-class) model last run — re-run ONLY these on sonnet;
// the 7 that completed stay cached (their opts are unchanged -> cache hit).
const FAILED_SCAN = new Set([
  'key-master-secrets', 'csrf-tokens', 'password-hashing', 'webhook-per-provider-deep',
  'jwt-token-format', 'rng-exhaustive', 'secret-hygiene-storage', 'dead-orphaned-crypto-code',
  'frontend-wasm-token-crypto', 'constant-time-correctness', 'logging-telemetry-leak',
])

const perDimension = await pipeline(
  DIMENSIONS,
  (d) => {
    const scanOpts = { label: `scan:${d.key}`, phase: 'Delta Scan', schema: FINDINGS_SCHEMA }
    if (FAILED_SCAN.has(d.key)) scanOpts.model = 'sonnet'
    return agent(
      `${COMMON}\n\n${PRIOR}\n\n${d.prompt}\n\nReturn structured findings. Set is_new_vs_prior=true for issues NOT already in the prior list above. Be exhaustive within this dimension.`,
      scanOpts
    )
  },
  (scan, d) => {
    if (!scan || !Array.isArray(scan.findings) || scan.findings.length === 0) {
      return { dimension: d.key, summary: (scan && scan.summary) || 'no findings', verified: [] }
    }
    return parallel(scan.findings.map((f) => () =>
      agent(
        `${COMMON}\n\nAdversarially VERIFY this finding against the ACTUAL source (Read the cited file & lines; quote real code). Default to REFUTED if the code does not support it.\n\nFINDING UNDER REVIEW (dimension ${d.key}):\n${JSON.stringify(f)}\n\nDetermine: confirmed (true/false), the ACTUAL CWE, ADJUSTED severity, real exploitability, concrete evidence lines, AND whether it is genuinely NEW vs the prior 88-finding list (is_new_vs_prior) with the overlapping prior ID if not new.`,
        { label: `verify:${d.key}`, phase: 'Adversarial Verify', schema: VERDICT_SCHEMA, model: 'sonnet' }
      )
        .then((v) => ({ ...f, dimension: d.key, verdict: v }))
        .catch(() => ({ ...f, dimension: d.key, verdict: null }))
    )).then((verified) => ({ dimension: d.key, summary: scan.summary || '', verified }))
  }
)

// Flatten confirmed findings
const allVerified = perDimension
  .filter(Boolean)
  .flatMap((p) => (p.verified || []).filter((x) => x.verdict && x.verdict.confirmed))

log(`Scan+verify complete: ${allVerified.length} confirmed findings across ${perDimension.filter(Boolean).length} dimensions.`)

// ---- Phase 3: three completeness critics (read the codebase themselves) ----
phase('Completeness Critics')

const confirmedBrief = allVerified.map((f) => `[${f.verdict.adjusted_severity}] ${f.dimension}: ${f.title} (${f.file}) [new=${f.verdict.is_new_vs_prior}]`).join('\n')

const [coverageCritic, deltaCritic, missingCritic] = await parallel([
  () => agent(
    `${COMMON}\n\n${PRIOR}\n\nYou are a COMPLETENESS CRITIC (dimension coverage). Here are the dimensions scanned: ${DIMENSIONS.map((d) => d.key).join(', ')}.\nHere are the CONFIRMED findings so far:\n${confirmedBrief}\n\nIndependently survey the crypto surface of the codebase (Glob/Grep/Read backend/api/src, frontend, Cargo.toml, traefik, docker, migrations) and judge: which crypto dimensions/sub-dimensions were COVERED, which were MISSED entirely, which were UNDER-covered. List NEW gaps not in the findings above. Score completeness 0-100. Be a skeptic — assume something was missed.`,
    { label: 'critic:coverage', phase: 'Completeness Critics', schema: CRITIC_SCHEMA, model: 'sonnet' }
  ),
  () => agent(
    `${COMMON}\n\n${PRIOR}\n\nYou are a DELTA CRITIC. Given the confirmed findings above:\n${confirmedBrief}\n\nClassify each as (a) genuinely NEW (not in prior 88), (b) STRENGTHENS or CORRECTS a prior finding (e.g. more precise, or refutes a prior claim), or (c) DUPLICATE of a prior finding. Pay special attention to findings that CORRECT the prior report (e.g. sha1 IS used in twofa, modulo bias in backup codes contradicting CRYP-RNG-007, STARTTLS is Tls::Required). List prior findings that this re-audit INVALIDATES or CORRECTS. List duplicates to drop.`,
    { label: 'critic:delta', phase: 'Completeness Critics', schema: CRITIC_SCHEMA, model: 'sonnet' }
  ),
  () => agent(
    `${COMMON}\n\n${PRIOR}\n\nYou are a MISSING-AREAS CRITIC. Independently explore the codebase (do NOT rely only on the findings) and answer: what crypto-relevant code/area has NOT been examined by any scanner? Examples to check: S3/RustFS presigned URLs, image/media upload integrity, search (quickwit) credentials, rate-limiter key hashing, cache key predictability, IdP logout/token-revocation, cookie SameSite interaction with OAuth redirects, HSTS/CSP nonces, open redirects, SSRF in URL fetches, magic-link/remember-me tokens, API rate-limit-token crypto, websocket auth. List every missing/under-covered area with the specific files to check and a NEW gap if you can find one by reading.`,
    { label: 'critic:missing', phase: 'Completeness Critics', schema: CRITIC_SCHEMA, model: 'sonnet' }
  ),
])

// ---- Phase 4: targeted resweep on critic-flagged gaps ----
phase('Targeted Resweep')

const gapTargets = Array.from(new Set([
  ...((coverageCritic && coverageCritic.missing_dimensions) || []),
  ...((coverageCritic && coverageCritic.under_covered) || []),
  ...((missingCritic && missingCritic.missing_dimensions) || []),
  ...((missingCritic && missingCritic.under_covered) || []),
  ...((coverageCritic && coverageCritic.new_gaps || []).map((g) => g.dimension + ' :: ' + g.title)),
  ...((missingCritic && missingCritic.new_gaps || []).map((g) => g.dimension + ' :: ' + g.title)),
])).filter(Boolean).slice(0, 12)

log(`Critics flagged ${gapTargets.length} gap areas; launching targeted resweeps.`)

const resweepRaw = await parallel(gapTargets.map((g) => () =>
  agent(
    `${COMMON}\n\n${PRIOR}\n\nA completeness critic flagged this area as MISSED or UNDER-COVERED:\n${g}\n\nExhaustively scan it NOW (Read the relevant files). Return only GENUINELY NEW findings not in the prior list and not already found. If the area is clean, return an empty findings array with a summary explaining what you checked.`,
    { label: `resweep:${String(g).slice(0, 24)}`, phase: 'Targeted Resweep', schema: FINDINGS_SCHEMA, model: 'sonnet' }
  ).catch(() => ({ dimension: String(g), findings: [] }))
))

// verify resweep findings
const resweepVerified = await parallel(
  resweepRaw.filter(Boolean).flatMap((r) => (r.findings || []).map((f) => () =>
    agent(
      `${COMMON}\n\nAdversarially VERIFY this resweep finding against ACTUAL source. Default to REFUTED if unsupported.\n${JSON.stringify(f)}\n\nReturn confirmed/CWE/severity/is_new_vs_prior/evidence.`,
      { label: 'verify:resweep', phase: 'Targeted Resweep', schema: VERDICT_SCHEMA, model: 'sonnet' }
    )
      .then((v) => ({ ...f, dimension: r.dimension, verdict: v }))
      .catch(() => ({ ...f, dimension: r.dimension, verdict: null }))
  ))
)
const resweepConfirmed = resweepVerified.filter(Boolean).filter((x) => x.verdict && x.verdict.confirmed)
log(`Resweep verified: ${resweepConfirmed.length} additional confirmed findings.`)

// ---- Phase 5: synthesis ----
phase('Synthesis')

const criticNewGaps = []
for (const c of [coverageCritic, deltaCritic, missingCritic]) {
  if (c && Array.isArray(c.new_gaps)) criticNewGaps.push(...c.new_gaps)
}

const everythingForSynth = [
  ...allVerified.map((f) => ({ dimension: f.dimension, title: f.title, severity: f.verdict.adjusted_severity, cwe: f.verdict.actual_cwe, file: f.file, line: f.line, description: f.description, is_new: f.verdict.is_new_vs_prior, overlap: f.verdict.overlap_with_prior, reasoning: f.verdict.reasoning })),
  ...resweepConfirmed.map((f) => ({ dimension: f.dimension, title: f.title, severity: f.verdict.adjusted_severity, cwe: f.verdict.actual_cwe, file: f.file, line: f.line, description: f.description, is_new: f.verdict.is_new_vs_prior, overlap: f.verdict.overlap_with_prior, reasoning: f.verdict.reasoning })),
]

const synthesis = await agent(
  `${COMMON}\n\n${PRIOR}\n\nYou are the SYNTHESIS agent. Produce the final cryptographic re-audit result from the verified findings below.\n\nVERIFIED FINDINGS (scan + resweep):\n${JSON.stringify(everythingForSynth, null, 0)}\n\nCRITIC-FLAGGED NEW GAPS (verify-before-trust; drop any that duplicate verified findings):\n${JSON.stringify(criticNewGaps, null, 0)}\n\nCRITIC COMPLETENESS SCORES: coverage=${coverageCritic && coverageCritic.completeness_score}, delta=(n/a), missing=${missingCritic && missingCritic.completeness_score}.\n\nTasks:\n1. DEDUPE findings that are the same issue from different scanners.\n2. Assign stable IDs of the form CRYP2-XXX (NEW findings) and CRYP2-CORR-XX (corrections to prior). Classification per finding: new | strengthening_or_correcting_prior | duplicate_confirming_prior.\n3. Produce severity_breakdown, top_new_findings (the most important genuinely-new ones, with why_new), genuinely_missed_areas, prior_corrections (list every prior claim this re-audit corrects or refutes, e.g. sha1 IS used, modulo bias EXISTS, STARTTLS is Tls::Required), completeness_vs_prior (how much was newly covered vs still missing), and all_findings.\n4. Write a candid executive_summary stating whether the prior 88-finding report missed anything material.`,
  { label: 'synthesis', phase: 'Synthesis', schema: SYNTH_SCHEMA, model: 'sonnet' }
)

return {
  confirmed_scan_findings: allVerified.length,
  confirmed_resweep_findings: resweepConfirmed.length,
  total_confirmed: allVerified.length + resweepConfirmed.length,
  dimensions_scanned: DIMENSIONS.length,
  critics: { coverage: coverageCritic, delta: deltaCritic, missing: missingCritic },
  synthesis,
  // raw per-dimension detail for the report writer:
  detail: perDimension.filter(Boolean).map((p) => ({
    dimension: p.dimension,
    summary: p.summary,
    findings: (p.verified || []).filter((x) => x.verdict && x.verdict.confirmed).map((f) => ({
      title: f.title, severity: f.verdict.adjusted_severity, cwe: f.verdict.actual_cwe,
      file: f.file, line: f.line, description: f.description, why_crypto: f.why_crypto,
      evidence: f.evidence, is_new: f.verdict.is_new_vs_prior, overlap: f.verdict.overlap_with_prior,
      exploitability: f.verdict.real_exploitability, reasoning: f.verdict.reasoning,
    })),
  })),
  resweep_detail: resweepConfirmed.map((f) => ({
    title: f.title, severity: f.verdict.adjusted_severity, cwe: f.verdict.actual_cwe,
    file: f.file, line: f.line, description: f.description, is_new: f.verdict.is_new_vs_prior,
    reasoning: f.verdict.reasoning,
  })),
}
