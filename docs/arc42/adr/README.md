# Architecture Decision Records

This directory holds the project's individual architecture decision records.
The chapter-09 register at
[`../09-architecture-decisions.md`](../09-architecture-decisions.md) links to
the same records and is the primary index.

| ADR | Title | Status | Date | File |
|---|---|---|---|---|
| ADR-0001 | Query digest in the model-cassette key | accepted | 2026-10-27 | [ADR-0001-query-digest-model-cassette-key.md](ADR-0001-query-digest-model-cassette-key.md) |

## Storage

- Active records live at `ADR-NNNN-short-slug.md`.
- Historical records live at `archive/ADR-NNNN-short-slug.md`; preserve the ID
  and update this index and the chapter-09 register when a record moves.
- Each record is copied from `adr-template.md` and tailored; the template is
  not a decision.
- Individual files are created manually; enabling the addon does not scaffold
  them.

## Qualification

Before adding a record, apply the complete qualification gate from
`givn instructions arc42-docs`. A decision that fails a mandatory dimension is
not an ADR and is routed once to its canonical artifact; it is not copied into
this directory.
