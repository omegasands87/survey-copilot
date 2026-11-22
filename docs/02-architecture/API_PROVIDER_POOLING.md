# AI Provider and API Key Pooling

**Status:** Normative technical specification for v1.0  
**Primary provider:** Google Gemini  
**Extensibility:** Any compatible provider, custom endpoint, and model

## 1. User outcome

The user can configure multiple legitimately authorized provider credentials (including separate keys where permitted by the provider) and the application automatically selects an eligible configuration. Pooling is for operational resilience and user-managed provider configurations, never for evading free-tier limits, project/account quotas, billing controls, or provider terms. The user should not need to understand transient rate-limit mechanics during normal use.

## 2. Provider configuration

Each provider configuration contains:
- Stable local ID and display name.
- Provider type: Gemini, OpenAI-compatible, Anthropic-compatible, or Custom HTTP adapter.
- Base URL / endpoint when applicable.
- API protocol / adapter type.
- Model identifier.
- Secret reference stored in Windows Credential Manager.
- Enabled/disabled state.
- Priority and optional weight.
- Per-key request rate and concurrency limits when known.
- Optional daily/minute quota hints entered by the user.
- Created/updated timestamps.

Never store API key values in SQLite, frontend state persistence, logs, crash reports, exported settings, or diagnostic bundles. Store only an opaque credential reference in SQLite. Mask keys after entry.

## 3. Pool behavior

Default strategy: adaptive availability-aware rotation.

1. Build the eligible pool from enabled configurations with a valid credential reference.
2. Exclude configurations in cooldown, disabled by the user, or known to be exhausted.
3. Prefer the configuration with the lowest recent utilization and suitable priority.
4. Apply per-key concurrency and rate limits before dispatch.
5. On success, update health, latency, and usage observations.
6. On transient rate limit (HTTP 429), honor Retry-After when supplied, otherwise apply bounded exponential backoff with jitter; mark that key cooling down and try another eligible key.
7. On quota exhaustion, mark the key exhausted until its configured reset window or a successful later probe.
8. On transient network/server failure, retry according to the retry policy and then try another eligible configuration.
9. On authentication/permission failure (typically 401/403), do not repeatedly retry; mark configuration as needing attention and continue to another eligible key.
10. On invalid request/model/schema errors, do not rotate blindly across keys; report a configuration/request error.
11. If every configuration is unavailable, show one concise status and offer Retry. Preserve the question and profile context locally.

The pool must never send the same request concurrently to multiple keys merely to race for a response. Key rotation is for resilience across legitimately configured credentials; it must not be used to evade provider quotas, account limits, billing controls, or terms of service. Respect aggregate project/account limits and stop routing when the provider indicates an account-level restriction.

## 4. Retry policy

- Retry only idempotent generation requests, or requests with a provider-supported idempotency mechanism.
- Maximum attempts per user action: configurable, default 3 total provider attempts across the pool.
- Respect provider Retry-After.
- Exponential backoff with jitter for transient failures.
- User can choose automatic retry, retry delay, and maximum attempts.
- Cancellation must stop queued retries and abort in-flight requests where supported.
- Avoid retrying content-policy refusals, malformed user input, invalid model names, or deterministic 4xx errors.
- Never retry indefinitely.

## 5. Load balancing

Supported strategies:
- Adaptive (default): select based on health, cooldown, utilization, priority, and latency.
- Round robin: rotate through eligible keys.
- Weighted: distribute requests by user-defined weight.
- Priority: exhaust higher-priority configurations before lower-priority ones.

A user may group keys by provider/model compatibility. A request may only be routed to configurations that support the requested operation and compatible schema.

## 6. Custom provider support

The app must allow a user to define:
- Provider name.
- API base URL and request path.
- Authentication method and header name.
- Request/response adapter type.
- Model name and optional request parameters.
- Timeout and rate-limit hints.

For security, custom HTTP adapters must use HTTPS by default. Plain HTTP is allowed only for localhost endpoints after explicit confirmation. Block file:, javascript:, data:, and arbitrary non-HTTP schemes. Do not allow a provider response to invoke tools, execute code, or access local files.

Arbitrary APIs cannot be assumed compatible. The UI must distinguish built-in adapters from custom adapters and validate the connection with a non-sensitive test request before use. Do not provide a general-purpose arbitrary code/plugin execution mechanism in v1.0.

## 7. Gemini adapter

Gemini is the default provider. The adapter must:
- Support user-selected Gemini models available to the user's API project.
- Keep the API key in the OS credential store.
- Parse provider rate-limit/quota responses into normalized error categories.
- Support structured JSON output where the selected model/API supports it.
- Preserve provider request IDs and usage metadata where available, without logging prompt contents.
- Avoid hardcoding model availability; let the user choose or refresh model IDs where supported.

## 8. Normalized error model

- rate_limited
- quota_exhausted
- authentication_failed
- permission_denied
- model_unavailable
- invalid_request
- content_refused
- timeout
- network_error
- provider_server_error
- response_parse_error
- cancelled
- unknown

Each failure records provider config ID, category, timestamp, retryability, and safe diagnostic metadata. Never persist the key or raw personal prompt in an error record.

## 9. Settings UX

Provide a simple provider screen:
- Add API
- Provider type
- Endpoint / model
- API key input
- Test connection
- Enable / disable
- Set priority / weight
- Show current health: Ready, Cooling down, Quota reached, Needs attention, Disabled
- Remove configuration

Advanced settings may expose strategy, retry limit, concurrency, timeout, and rate-limit hints. Defaults should work without tuning.

## 10. Acceptance criteria

- User can add multiple Gemini keys and test them independently.
- Secrets are stored only in Windows Credential Manager.
- Rate-limited key is cooled down and another eligible key is attempted.
- Authentication failure does not create an infinite retry loop.
- Adaptive, round-robin, weighted, and priority strategies work deterministically.
- User cancellation stops retries.
- If all keys fail, question and extracted screen content remain available.
- Custom HTTPS endpoint and model can be configured without changing application code when using a supported adapter.
- Unsupported custom protocol is rejected with a clear explanation.
- Logs and exports contain no API key values or raw prompts by default.

## 11. Implemented v1.0 Pool Architecture (SC-022)

The Provider Pool implementation is modularized across backend and frontend layers:

- **Domain Model (`src-tauri/src/domain/pool.rs`)**:
  - `PoolStrategy`: `adaptive`, `round_robin`, `weighted`, `priority`.
  - `PoolSettings`: global pool parameters (`strategy`, `max_attempts`, `auto_retry`, `retry_base_delay_ms`, `max_retry_delay_ms`).
  - `ProviderHealthState`: `ready`, `cooling_down`, `quota_exhausted`, `needs_attention`, `disabled`.
  - `ProviderHealthInfo`: health metrics, latency observations, consecutive failures, and active concurrency.
  - `ProviderAttemptLog` and `PoolExecutionResult`: structured result contract strictly preserving user request payloads on failure.
  - `CancellationToken` and `TimeSource` (with `FakeTimeSource` for deterministic tests).

- **Service Layer (`src-tauri/src/services/provider_pool.rs`)**:
  - `ProviderPoolService`: RAII concurrency tracking (`ConcurrencyGuard`), strategy-based candidate selection, cooldown calculation with bounded exponential backoff and `Retry-After` enforcement.
  - Account/Project Quota Exhaustion Policy (AT-034): immediate abort of retries upon detecting account/project quota exhaustion (`QuotaExhausted`).
  - Non-retryable error policy (AT-035): immediate halt on `InvalidRequest`, `ContentRefused`, and permanent `AuthenticationFailed`.
  - Per-credential rate limit policy (AT-033): transient cooldown and bounded failover to next eligible key.
  - Cancellation policy (AT-036): immediate cessation of queued retries.

- **Persistence Layer (`src-tauri/src/repository/pool_repo.rs`)**:
  - Parameterized SQLite operations on `provider_pool_settings` and `provider_health`.
  - Automatic expiration and reset of past cooldowns (`reset_cooldown_if_expired`).

- **Typed IPC Commands (`src-tauri/src/commands/provider_pool.rs`)**:
  - `provider_pool_get_settings`
  - `provider_pool_update_settings`
  - `provider_pool_list_health`
  - `provider_pool_execute_request`

- **Frontend & Verification**:
  - Typed definitions: `src/types/providerPool.ts`.
  - Service: `src/services/providerPoolService.ts`.
  - Settings UI: `src/components/views/SettingsView.tsx` with pool strategy selector, retry parameters, and provider health badges.
  - Automated tests: `src-tauri/tests/provider_pool_tests.rs` and `tests/provider_pool.test.ts`.
