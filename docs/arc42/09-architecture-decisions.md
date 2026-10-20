# 9. Architecture Decisions

This file is the chapter-09 register/index for architecture decisions. It is
not an ADR body and not a container for multiple ADRs. One architecturally
significant decision gets exactly one ADR, stored as a separate Markdown file
under `docs/arc42/adr/ADR-NNNN-short-slug.md`.

The ADR directory has its own index at `docs/arc42/adr/README.md`. Create and
maintain the directory, files, and index entries manually; this asset does not
claim automatic scaffolding for individual ADRs. The current binary uses the
fixed `docs/arc42/` root and does not implement an `arc42.output_dir` override.

## Register

Maintain one row or link per ADR. Include the ADR ID, short title, current
status, date, and a link to the individual file. Do not paste an ADR body or a
second MADR section into this file.

| ADR | Title | Status | Date | File |
|---|---|---|---|---|

## Decision register status (benchmark-gate, 2026-10-20)

No architecture decision record is registered for the benchmark harness. The
harness language, workspace membership, recording/replay boundary, and
no-container isolation have real alternatives, but they are single-component,
single-change decisions with no durable product boundary; their complete
rationale is owned by `givn/changes/benchmark-gate/design.md` (after archive,
the design is the canonical change record).

`ADR action: NONE`

```text
qualification: NOT_QUALIFIED
alternatives: PASS
architectural_impact: FAIL
durable_consequence: FAIL
lower_level_artifact: FAIL
existing_adr_check: PASS
must_be_shared: SUPPORTING
routing: CANONICAL_ARTIFACT
canonical_artifact: design.md
target_adr: null
replacement_adr: null
```

Existing-ADR check: the register above and `docs/arc42/adr/README.md` were
searched; neither holds an ADR that covers the benchmark harness, its language,
workspace, recorder, or isolation. There is no duplicate or refinement to
amend and no record to supersede.

The decision is not a product-behavior decision owned by a use case either: the
benchmark is a project gate outside product runtime, and its observable
behaviour is owned by the `benchmark-comparison` Gherkin specification.


## Individual ADR contract

Before creating a file or adding a register entry, read and pass the complete
qualification procedure from the resolved Arc42 addon instruction. This chapter
records the register contract only; it does not duplicate the qualification
formula, LLM verdict schema, or routing values. A failed candidate is routed
exactly once to its canonical feature, design, Arc42, process,
project-documentation, or code location. A refinement amends an existing ADR;
it does not create a parallel file.

Each individual file is written and tailored from the ADR template at
`docs/arc42/adr/adr-template.md` (scaffolded from
`addons/arc42-docs/chapters/adr/adr-template.md`). That template is
self-contained: it embeds the frontmatter contract, the Qualification gate, the
LLM qualification verdict, the MADR checklist evidence, and every required MADR
section, so it is authoritative at authoring time even when the resolved Arc42
instruction is not loaded. Use it to produce the complete MADR structure,
including `## Qualification`, `## Context and Problem Statement`,
`## Decision Drivers`, `## Considered Options`, `## Decision Outcome`,
`### Consequences`, `### Confirmation`, `## Pros and Cons of the Options`, and
`## More Information`, with real frontmatter and an `ADR-QUALIFICATION: PASS`
verdict whose route-specific target fields are valid.
Allowed statuses are `proposed`, `rejected`, `accepted`, `deprecated`, and
`superseded by ADR-NNNN`. A superseded status must name a concrete existing
replacement ID, such as `superseded by ADR-027` or `superseded by ADR-0042`;
placeholders are invalid in completed ADRs. Keep `docs/arc42/adr/README.md` and
this register linked to the current or historical file.

## Archive

An archived historical ADR keeps its ID and filename and is moved to
`docs/arc42/adr/archive/ADR-NNNN-short-slug.md`. Update both indexes when the
path changes. Do not move ADRs into `givn/archive/<change>/`; that path is for
change artifacts, not durable architecture decisions.
Non-ADR material is not archived as an ADR. Move useful information once to its
canonical feature, design, Arc42, process, project-documentation, or code
location, and remove the redundant record.
