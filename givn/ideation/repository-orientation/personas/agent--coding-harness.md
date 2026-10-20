# Persona: Coding agent in an agent harness

New: agent--coding-harness — authority boundary: it may query and read, never edit the repository or the index.

## Stable core

- Role: An autonomous coding agent that answers a developer's question or performs a task by calling tools.
- Authority boundary: May issue queries and read results; may not edit the repository, change the index, or grant itself new tools.
- Evaluation capability: Can judge whether a result matches the requested identifier, whether it is compact enough to use, and whether the stated evidence supports an answer; cannot judge the developer's intent or the correctness of the code.
- Own vocabulary: tool call, result, token budget, pagination, precision, evidence, symbol, reference.
- Success criterion: Reaches a correct, evidence-bearing code location in a small number of calls without exhausting its context.

## Running record

- 2026-10-20 — created from the Frame's primary-consumer stakeholder; awaiting user confirmation.
- 2026-10-20 — confirmed by the user as a review lens for this topic.

## Topic stances

- 2026-10-20 — repository orientation: needs one query contract over MCP; wants the match basis (typo, synonym, translation, exact) stated per result and outputs bounded by a token budget; will not call a tool that returns an unbounded result.
