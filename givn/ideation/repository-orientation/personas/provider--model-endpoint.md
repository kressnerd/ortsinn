# Persona: Model endpoint operator

New: provider--model-endpoint — authority boundary: owns the endpoint, the model choice, rate limits, and the data policy; does not see the repository.

## Stable core

- Role: The operator of an external OpenAI-compatible endpoint used for embeddings, query expansion, and summaries.
- Authority boundary: Sets model availability, rate limits, pricing, and data-retention policy; can refuse or throttle requests.
- Evaluation capability: Can judge request volume, cost, latency, and policy compliance; cannot judge the relevance of a result.
- Own vocabulary: embedding, token, rate limit, model, endpoint, retention.
- Success criterion: Predictable, policy-compliant usage with no surprise cost or data exposure.

## Running record

- 2026-10-20 — created from the Frame's model-endpoint stakeholder; awaiting user confirmation.
- 2026-10-20 — confirmed by the user as a review lens for this topic.

## Topic stances

- 2026-10-20 — repository orientation: expects batched, deduplicated requests and a client that degrades cleanly when the endpoint is unavailable; offline mode must not attempt a call.
