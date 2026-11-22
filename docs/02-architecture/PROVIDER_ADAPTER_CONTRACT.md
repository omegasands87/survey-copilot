# Provider Adapter Contract

## 1. Purpose

Keep Gemini-specific and custom-provider protocol details outside the answer engine. Domain services depend only on normalized request, response, and error types.

## 2. Normalized request

- request_id: UUID
- operation: question_analysis | question_explanation | visual_interpretation | survey_criteria_extraction
- model_id
- system_instruction
- user_payload
- response_schema
- generation_options
- timeout_ms
- cancellation_token

The provider receives only the minimized content needed for the current operation.

## 3. Normalized response

- request_id
- provider_request_id (optional)
- provider_id
- model_id
- text or structured_json
- finish_reason
- input_tokens (optional)
- output_tokens (optional)
- latency_ms
- received_at

The adapter does not validate persona facts or make eligibility decisions. The application validates structured output and evidence references.

## 4. Normalized errors

Map provider-specific failures to:
rate_limited, quota_exhausted, authentication_failed, permission_denied, model_unavailable, invalid_request, content_refused, timeout, network_error, provider_server_error, response_parse_error, cancelled, unknown.

Include retryable, retry_after_ms, provider_request_id, and safe diagnostic fields where available. Never include API key values or full request payloads.

## 5. Adapter operations

- validate_config
- test_connection
- list_models (optional capability)
- generate_structured
- cancel (optional capability)
- normalize_error

Custom adapters must declare supported operations, schema mode, model compatibility, and cancellation capability.

## 6. Provider pool interface

The pool selects exactly one eligible configuration for an attempt. It applies concurrency and cooldown controls before calling the adapter. It updates health after the response and decides whether to fail over based on the normalized error category.

The pool must not retry deterministic invalid requests or content refusals. It must bound all retries and respect cancellation.

## 7. Security

- API key injection occurs only inside the native provider service.
- Redirects must not forward credentials to an unapproved origin.
- Custom endpoints must be HTTPS, except explicitly confirmed localhost HTTP.
- Response content is untrusted data.
- Provider adapters cannot execute code or invoke local tools.
