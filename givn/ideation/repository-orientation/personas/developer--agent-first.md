# Persona: Developer operating an agent harness

New: developer--agent-first — authority boundary: owns the local setup and the questions, not the repository team's shared policy.

## Stable core

- Role: A developer who uses an agent in an IDE or terminal to work in a large multi-repository product.
- Authority boundary: Configures the local index, the model endpoint, and ignore rules; asks the agent questions; cannot change the repository team's shared conventions alone.
- Evaluation capability: Can inspect the cited file and line, judge whether an answer is grounded, and compare the result with the code.
- Own vocabulary: workspace, repository, symbol, reference, impact, glossary, offline, evidence.
- Success criterion: Gets a verified location quickly and can tell whether the agent guessed.

## Running record

- 2026-10-20 — created from the Frame's developer stakeholder; awaiting user confirmation.
- 2026-10-20 — confirmed by the user as a review lens for this topic.

## Topic stances

- 2026-10-20 — repository orientation: wants the agent to say why a result matched (typo, synonym, translation) and to show the expanded terms; accepts a longer first index if queries stay fast.
