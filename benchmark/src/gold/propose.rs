//! Draft gold proposal. A draft is written under `gold/drafts/` and is never
//! loaded or scored: only a reviewed file the manifest references reaches
//! `Gold::load`. The maintainer reviews a draft, moves it into the reviewed
//! tree, and adds its manifest entry with `reviewed_by`.
//!
//! A proposal is a function of two reviewed inputs: the seed file (query
//! intent) and the fixed checkout (definition line ranges and documentation).
//! The definition scan is textual and deliberately conservative: a seed whose
//! body extent or doc block cannot be established fails the proposal instead
//! of producing a guessed line range.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::seeds::{Seed, SeedFile, SeedLanguage};

/// Directory, relative to the gold root, that holds unreviewed proposals.
pub const DRAFTS_DIR: &str = "drafts";

/// Directory, relative to the gold root, that holds the reviewed seed files.
pub const SEEDS_DIR: &str = "seeds";

/// Metric names in report order.
pub const METRICS: [&str; 5] = [
    "typo",
    "natural_language",
    "explanation",
    "concept",
    "concept_equivalence",
];

/// Typo tasks that must find nothing. They keep the no-match rate honest
/// instead of measuring only queries that have an answer.
pub const NO_MATCH_TASKS: u64 = 5;

/// Queries for typo tasks with no expected location. They are plausible
/// symbol spellings that no reference repository defines.
const NO_MATCH_QUERIES: [&str; 5] = [
    "frobnicateSourceTree",
    "instantiateFluxCapacitor",
    "readTelemetryBeacon",
    "computeQuantumEntropy",
    "reserializeWidgetRegistry",
];

/// Path of an unreviewed draft file for one repository and metric.
pub fn draft_path(gold_root: &Path, repository: &str, metric: &str) -> PathBuf {
    gold_root
        .join(DRAFTS_DIR)
        .join(repository)
        .join(format!("{metric}.jsonl"))
}

/// Path of a reviewed gold file for one repository and metric.
pub fn reviewed_path(gold_root: &Path, repository: &str, metric: &str) -> PathBuf {
    gold_root
        .join(repository)
        .join(format!("{metric}.jsonl"))
}

/// Write unreviewed draft tasks. The reviewed tree is never touched, and the
/// manifest is never updated, so a draft cannot silently join a gold set.
pub fn write_draft(
    gold_root: &Path,
    repository: &str,
    metric: &str,
    tasks: &[serde_json::Value],
) -> Result<PathBuf> {
    let path = draft_path(gold_root, repository, metric);
    write_tasks(&path, tasks)?;
    Ok(path)
}

/// Write reviewed gold tasks. Called only with the reviewed seeds and the
/// fixed checkout in hand; the manifest entry is authored separately.
pub fn write_reviewed(
    gold_root: &Path,
    repository: &str,
    metric: &str,
    tasks: &[serde_json::Value],
) -> Result<PathBuf> {
    let path = reviewed_path(gold_root, repository, metric);
    write_tasks(&path, tasks)?;
    Ok(path)
}

fn write_tasks(path: &Path, tasks: &[serde_json::Value]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create gold directory {}", parent.display()))?;
    }
    let mut text = String::new();
    for task in tasks {
        text.push_str(&serde_json::to_string(task).context("serialize gold task")?);
        text.push('\n');
    }
    fs::write(path, text).with_context(|| format!("write gold file {}", path.display()))?;
    Ok(())
}

/// A definition resolved in a fixed checkout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub path: String,
    pub symbol: String,
    pub start_line: u32,
    pub end_line: u32,
    pub doc_start: u32,
    pub doc_end: u32,
    pub doc_text: String,
    /// Reviewed query overrides from the seed file.
    pub concept: Option<String>,
    pub natural_language: Option<String>,
    pub equivalence: Option<String>,
    pub typo: Option<String>,
}

/// Resolve every seed against the checkout. A seed that cannot be resolved
/// with a documented definition is a review error, named by path and symbol.
pub fn resolve_seeds(root: &Path, seed_file: &SeedFile) -> Result<Vec<Definition>> {
    seed_file
        .seeds
        .iter()
        .map(|seed| resolve_seed(root, seed_file.language, seed))
        .collect()
}

fn resolve_seed(root: &Path, language: SeedLanguage, seed: &Seed) -> Result<Definition> {
    let path = root.join(&seed.path);
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read seed definition {}", path.display()))?;
    let lines: Vec<&str> = text.lines().collect();
    let mut rejected = Vec::new();
    for index in definition_lines(&lines, language, &seed.symbol)? {
        let Some(end) = body_end(&lines, language, index)? else {
            rejected.push(format!("line {}: declaration has no body", index + 1));
            continue;
        };
        let Some((doc_start, doc_end, doc_text)) = documentation(&lines, language, index) else {
            rejected.push(format!("line {}: no doc comment", index + 1));
            continue;
        };
        if doc_text.trim().is_empty() {
            rejected.push(format!("line {}: doc comment has no prose", index + 1));
            continue;
        }
        return Ok(Definition {
            path: seed.path.clone(),
            symbol: seed.symbol.clone(),
            start_line: (index + 1) as u32,
            end_line: (end + 1) as u32,
            doc_start: (doc_start + 1) as u32,
            doc_end: (doc_end + 1) as u32,
            doc_text,
            concept: seed.concept.clone(),
            natural_language: seed.natural_language.clone(),
            equivalence: seed.equivalence.clone(),
            typo: seed.typo.clone(),
        });
    }
    bail!(
        "seed {} {} in {}: no documented definition ({})",
        seed.path,
        seed.symbol,
        path.display(),
        rejected.join("; ")
    )
}

/// Build the five reviewed gold files of one repository from its seeds.
/// Counts come from the manifest minima for the repository's power, so a
/// reviewed set always meets the published minimum by construction.
pub fn propose(
    seed_file: &SeedFile,
    definitions: &[Definition],
    counts: &BTreeMap<String, u64>,
    id_prefix: &str,
) -> Result<BTreeMap<String, Vec<serde_json::Value>>> {
    anyhow::ensure!(
        definitions.len() as u64 >= count(counts, "concept")?,
        "seed file {} has {} definitions, fewer than the {} concept tasks it must supply",
        seed_file.repository,
        definitions.len(),
        count(counts, "concept")?
    );
    let mut sets = BTreeMap::new();
    sets.insert("typo".to_string(), typo_tasks(definitions, counts, id_prefix)?);
    sets.insert(
        "natural_language".to_string(),
        natural_language_tasks(definitions, counts, id_prefix)?,
    );
    sets.insert(
        "explanation".to_string(),
        explanation_tasks(definitions, counts, id_prefix)?,
    );
    sets.insert("concept".to_string(), concept_tasks(definitions, counts, id_prefix)?);
    sets.insert(
        "concept_equivalence".to_string(),
        equivalence_tasks(definitions, counts, id_prefix)?,
    );
    for (metric, tasks) in &sets {
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for task in tasks {
            let query = task
                .get("query")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let id = task
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            if let Some(previous) = seen.insert(query, id) {
                bail!(
                    "{} {metric} tasks {previous} and {id} share the query {query:?}; add a reviewed override",
                    seed_file.repository
                );
            }
        }
    }
    Ok(sets)
}

fn count(counts: &BTreeMap<String, u64>, metric: &str) -> Result<u64> {
    counts
        .get(metric)
        .copied()
        .ok_or_else(|| anyhow::anyhow!("manifest gold minimums do not declare {metric}"))
}

fn definition_json(definition: &Definition) -> serde_json::Value {
    serde_json::json!({
        "path": definition.path,
        "start_line": definition.start_line,
        "end_line": definition.end_line,
        "symbol": definition.symbol,
    })
}

fn typo_tasks(
    definitions: &[Definition],
    counts: &BTreeMap<String, u64>,
    id_prefix: &str,
) -> Result<Vec<serde_json::Value>> {
    let total = count(counts, "typo")?;
    anyhow::ensure!(
        total > NO_MATCH_TASKS,
        "typo minimum {total} leaves no room for {NO_MATCH_TASKS} no-match tasks"
    );
    let targeted = total - NO_MATCH_TASKS;
    anyhow::ensure!(
        definitions.len() as u64 >= targeted,
        "typo needs {targeted} definitions, the seeds provide {}",
        definitions.len()
    );
    let mut tasks = Vec::new();
    for (index, definition) in definitions.iter().take(targeted as usize).enumerate() {
        tasks.push(serde_json::json!({
            "id": format!("{id_prefix}-typo-{:02}", index + 1),
            "query": typo_query(definition),
            "expected": [definition_json(definition)],
        }));
    }
    for (offset, query) in NO_MATCH_QUERIES.iter().enumerate() {
        tasks.push(serde_json::json!({
            "id": format!("{id_prefix}-typo-{:02}", targeted as usize + offset + 1),
            "query": query,
            "expected": [],
        }));
    }
    Ok(tasks)
}

fn natural_language_tasks(
    definitions: &[Definition],
    counts: &BTreeMap<String, u64>,
    id_prefix: &str,
) -> Result<Vec<serde_json::Value>> {
    let total = count(counts, "natural_language")?;
    anyhow::ensure!(
        definitions.len() as u64 >= total,
        "natural-language needs {total} definitions, the seeds provide {}",
        definitions.len()
    );
    Ok(definitions
        .iter()
        .take(total as usize)
        .enumerate()
        .map(|(index, definition)| {
            serde_json::json!({
                "id": format!("{id_prefix}-nl-{:02}", index + 1),
                "query": natural_language_query(definition),
                "expected": [definition_json(definition)],
            })
        })
        .collect())
}

fn explanation_tasks(
    definitions: &[Definition],
    counts: &BTreeMap<String, u64>,
    id_prefix: &str,
) -> Result<Vec<serde_json::Value>> {
    let total = count(counts, "explanation")?;
    anyhow::ensure!(
        definitions.len() as u64 >= total,
        "explanation needs {total} definitions, the seeds provide {}",
        definitions.len()
    );
    Ok(definitions
        .iter()
        .take(total as usize)
        .enumerate()
        .map(|(index, definition)| {
            serde_json::json!({
                "id": format!("{id_prefix}-exp-{:02}", index + 1),
                "query": format!(
                    "Explain what {} does in {} and cite the code that shows it.",
                    definition.symbol, definition.path
                ),
                "target": definition_json(definition),
                "required_facts": required_facts(definition),
                "acceptable_evidence": [{
                    "start_line": definition.doc_start,
                    "end_line": definition.end_line,
                }],
            })
        })
        .collect())
}

fn concept_tasks(
    definitions: &[Definition],
    counts: &BTreeMap<String, u64>,
    id_prefix: &str,
) -> Result<Vec<serde_json::Value>> {
    let total = count(counts, "concept")?;
    Ok(definitions
        .iter()
        .take(total as usize)
        .enumerate()
        .map(|(index, definition)| {
            serde_json::json!({
                "id": format!("{id_prefix}-concept-{:02}", index + 1),
                "query": concept_query(definition),
                "expected": [definition_json(definition)],
                "equivalence": false,
            })
        })
        .collect())
}

fn equivalence_tasks(
    definitions: &[Definition],
    counts: &BTreeMap<String, u64>,
    id_prefix: &str,
) -> Result<Vec<serde_json::Value>> {
    let total = count(counts, "concept_equivalence")?;
    anyhow::ensure!(
        definitions.len() as u64 >= total,
        "concept-equivalence needs {total} definitions, the seeds provide {}",
        definitions.len()
    );
    Ok(definitions
        .iter()
        .take(total as usize)
        .enumerate()
        .map(|(index, definition)| {
            serde_json::json!({
                "id": format!("{id_prefix}-ce-{:02}", index + 1),
                "query": equivalence_query(definition),
                "expected": [definition_json(definition)],
                "equivalence": true,
            })
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Query derivation
// ---------------------------------------------------------------------------

/// Verbs a symbol name may start with. The concept query drops the verb so it
/// reads as a concept, and the equivalence query restates it as a noun.
const VERBS: [&str; 32] = [
    "add", "bind", "can", "check", "compute", "copy", "create", "emit", "ensure", "find",
    "flatten", "get", "has", "index", "insert", "is", "load", "make", "normalize", "open",
    "parse", "read", "resolve", "set", "should", "skip", "sort", "take", "to", "validate",
    "walk", "write",
];

fn split_words(symbol: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_lower = false;
    for character in symbol.chars() {
        if character == '_' {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
            previous_lower = false;
            continue;
        }
        if character.is_uppercase() && previous_lower {
            words.push(current.clone());
            current.clear();
        }
        previous_lower = character.is_lowercase() || character.is_ascii_digit();
        current.push(character);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
        .into_iter()
        .map(|word| word.to_lowercase())
        .filter(|word| !word.is_empty())
        .collect()
}

fn split_verb(words: &[String]) -> (Option<String>, &[String]) {
    if words.len() >= 2 && words[0] == "for" && words[1] == "each" {
        return (Some("forEach".to_string()), &words[2..]);
    }
    match words.first() {
        Some(first) if VERBS.contains(&first.as_str()) && words.len() > 1 => {
            (Some(first.clone()), &words[1..])
        }
        _ => (None, words),
    }
}

fn noun_form(verb: &str) -> String {
    let noun = match verb {
        "is" | "check" | "validate" => "check",
        "get" | "find" => "lookup",
        "has" => "presence",
        "can" | "should" => "capability",
        "set" | "add" | "insert" => "assignment",
        "create" | "make" => "creation",
        "ensure" => "validation",
        "resolve" => "resolution",
        "walk" => "traversal",
        "forEach" => "iteration",
        "to" => "conversion",
        "parse" => "parsing",
        "bind" => "binding",
        "emit" => "emission",
        "read" | "load" => "reading",
        "write" => "writing",
        "open" => "opening",
        "normalize" => "normalization",
        "sort" => "ordering",
        "flatten" => "flattening",
        "skip" => "skipping",
        "take" => "extraction",
        "compute" => "computation",
        "copy" => "copying",
        "index" => "indexing",
        other => other,
    };
    noun.to_string()
}

fn humanized(symbol: &str) -> String {
    split_words(symbol).join(" ")
}

fn concept_query(definition: &Definition) -> String {
    if let Some(concept) = &definition.concept {
        return concept.clone();
    }
    let words = split_words(&definition.symbol);
    let (_, rest) = split_verb(&words);
    if rest.len() >= 2 || rest.first().is_some_and(|word| word.len() >= 6) {
        rest.join(" ")
    } else {
        humanized(&definition.symbol)
    }
}

fn equivalence_query(definition: &Definition) -> String {
    if let Some(equivalence) = &definition.equivalence {
        return equivalence.clone();
    }
    let words = split_words(&definition.symbol);
    let (verb, rest) = split_verb(&words);
    match verb {
        Some(verb) if !rest.is_empty() => format!("{} {}", rest.join(" "), noun_form(&verb)),
        _ => {
            let mut reversed = words;
            reversed.reverse();
            reversed.join(" ")
        }
    }
}

fn typo_query(definition: &Definition) -> String {
    if let Some(typo) = &definition.typo {
        return typo.clone();
    }
    let mut characters: Vec<char> = definition.symbol.chars().collect();
    let mut end = characters.len();
    while end > 0 && !characters[end - 1].is_ascii_alphabetic() {
        end -= 1;
    }
    if end >= 2 && characters[end - 1].is_ascii_alphabetic() {
        characters.swap(end - 1, end - 2);
    }
    characters.into_iter().collect()
}

fn natural_language_query(definition: &Definition) -> String {
    if let Some(query) = &definition.natural_language {
        return query.clone();
    }
    let sentence = first_sentence(&definition.doc_text);
    let sentence = strip_markdown(&sentence);
    if sentence.is_empty() {
        return format!("Where is {} defined?", definition.symbol);
    }
    let mut characters = sentence.chars();
    let first = characters
        .next()
        .map(|character| character.to_lowercase().to_string())
        .unwrap_or_default();
    format!("Where is the code that {first}{}?", characters.as_str())
}

/// Up to three distinctive prose words as alternatives groups. A correct
/// grounded explanation of the documented behaviour uses at least one form of
/// each group. Verbs, adverbs, and words that only repeat the symbol name are
/// not distinctive, so they are skipped.
fn required_facts(definition: &Definition) -> Vec<Vec<String>> {
    let sentence = first_sentence(&definition.doc_text);
    let symbol_parts: Vec<String> = split_words(&definition.symbol);
    let mut candidates: Vec<(String, usize)> = Vec::new();
    for (index, word) in sentence.split_whitespace().enumerate() {
        let cleaned = prose_word(word);
        let Some(cleaned) = cleaned else {
            continue;
        };
        if cleaned.len() < 5
            || STOP_WORDS.contains(&cleaned.as_str())
            || symbol_parts.contains(&cleaned)
            || cleaned.ends_with("ed")
            || cleaned.ends_with("ing")
            || cleaned.ends_with("ly")
        {
            continue;
        }
        if candidates.iter().any(|(seen, _)| *seen == cleaned) {
            continue;
        }
        candidates.push((cleaned, index));
    }
    candidates.sort_by(|left, right| right.0.len().cmp(&left.0.len()).then(left.1.cmp(&right.1)));
    let mut facts: Vec<Vec<String>> = candidates
        .into_iter()
        .take(3)
        .map(|(word, _)| forms(&word))
        .collect();
    if facts.len() < 2 {
        let mut extras: Vec<String> = Vec::new();
        for word in concept_query(definition).split_whitespace() {
            let Some(word) = prose_word(word) else {
                continue;
            };
            if word.len() >= 4
                && !STOP_WORDS.contains(&word.as_str())
                && !extras.contains(&word)
            {
                extras.push(word);
            }
        }
        for part in &symbol_parts {
            if !extras.contains(part) {
                extras.push(part.clone());
            }
        }
        for extra in extras {
            if facts
                .iter()
                .any(|group| group.iter().any(|form| form == &extra))
            {
                continue;
            }
            facts.push(vec![extra]);
            if facts.len() == 2 {
                break;
            }
        }
    }
    facts
}

/// A prose word, or `None` for a code span, punctuation, or a token that is
/// not purely alphabetic once surrounding punctuation is removed.
fn prose_word(word: &str) -> Option<String> {
    const PUNCTUATION: [char; 24] = [
        '`', '"', '\'', '.', ',', ';', ':', '(', ')', '[', ']', '{', '}', '<', '>', '/', '\\',
        '|', '*', '!', '=', '+', '#', '@',
    ];
    let trimmed = word.trim_matches(|character: char| PUNCTUATION.contains(&character));
    if trimmed.is_empty()
        || !trimmed
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        return None;
    }
    Some(trimmed.to_lowercase())
}

fn forms(word: &str) -> Vec<String> {
    if word.ends_with('s') {
        let mut forms = vec![word.to_string()];
        if word.len() >= 6 && !word.ends_with("ss") && !word.ends_with("us") {
            forms.push(word[..word.len() - 1].to_string());
        }
        return forms;
    }
    let mut forms = vec![word.to_string()];
    let consonant_before_y = word.len() > 4
        && word.ends_with('y')
        && !matches!(
            word.chars().nth(word.len() - 2),
            Some('a' | 'e' | 'i' | 'o' | 'u')
        );
    if consonant_before_y {
        forms.push(format!("{}ies", &word[..word.len() - 1]));
    } else if !word.ends_with("less") && !word.ends_with("ous") && !word.ends_with("ful") {
        forms.push(format!("{word}s"));
    }
    forms
}

const STOP_WORDS: [&str; 43] = [
    "about", "additional", "additionally", "adjacent", "after", "another", "before", "being",
    "between", "corresponding", "could", "equivalent", "given", "including", "lightweight",
    "matching", "other", "otherwise", "place", "places", "provided", "return", "returns",
    "several", "should", "similar", "simple", "specified", "their", "there", "these", "thing",
    "those", "truthy", "using", "value", "values", "where", "whether", "which", "while",
    "without", "would",
];

fn first_sentence(text: &str) -> String {
    let mut out = String::new();
    for character in text.chars() {
        if matches!(character, '.' | '?' | '!' | ':') && !out.trim().is_empty() {
            break;
        }
        out.push(character);
        if out.len() >= 180 {
            break;
        }
    }
    out.trim().trim_end_matches(['.', ',', ';']).to_string()
}

fn strip_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '`' => {}
            '[' => {
                // Drop a link label; keep its text.
                while let Some(next) = characters.next() {
                    if next == ']' {
                        break;
                    }
                    out.push(next);
                }
            }
            '*' | '#' | '_' => {}
            other => out.push(other),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Definition scanning
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanState {
    Code,
    BlockComment,
    Text(char),
    RawText(usize),
}

/// Line indexes whose declaration matches the seed symbol.
fn definition_lines(
    lines: &[&str],
    language: SeedLanguage,
    symbol: &str,
) -> Result<Vec<usize>> {
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if declaration_matches(line, language, symbol) {
            found.push(index);
        }
    }
    anyhow::ensure!(!found.is_empty(), "no declaration of {symbol} found");
    Ok(found)
}

fn declaration_matches(line: &str, language: SeedLanguage, symbol: &str) -> bool {
    let trimmed = line.trim_start();
    match language {
        SeedLanguage::TypeScript => {
            let Some(rest) = trimmed.strip_prefix("export function ") else {
                return false;
            };
            let Some(after) = rest.strip_prefix(symbol) else {
                return false;
            };
            let after = after.trim_start();
            after.starts_with('(') || after.starts_with('<')
        }
        SeedLanguage::Python => {
            let Some(rest) = trimmed
                .strip_prefix("def ")
                .or_else(|| trimmed.strip_prefix("class "))
            else {
                return false;
            };
            let Some(after) = rest.strip_prefix(symbol) else {
                return false;
            };
            let after = after.trim_start();
            after.starts_with('(') || after.starts_with(':')
        }
        SeedLanguage::Rust => {
            let Some(rest) = trimmed.strip_prefix("pub ") else {
                return false;
            };
            let mut rest = rest.trim_start();
            loop {
                let mut advanced = false;
                for prefix in ["const ", "unsafe ", "async ", "extern \"C\" "] {
                    if let Some(after) = rest.strip_prefix(prefix) {
                        rest = after.trim_start();
                        advanced = true;
                        break;
                    }
                }
                if !advanced {
                    break;
                }
            }
            let Some(after) = rest.strip_prefix("fn ") else {
                return false;
            };
            let Some(after) = after.strip_prefix(symbol) else {
                return false;
            };
            let after = after.trim_start();
            after.starts_with('(') || after.starts_with('<') || after.starts_with('\'')
        }
    }
}

/// Last line of the definition body, including a single-line body.
fn body_end(lines: &[&str], language: SeedLanguage, start: usize) -> Result<Option<usize>> {
    match language {
        SeedLanguage::Python => Ok(python_body_end(lines, start)),
        SeedLanguage::TypeScript => ts_body_end(lines, start),
        SeedLanguage::Rust => rust_body_end(lines, start),
    }
}

fn python_body_end(lines: &[&str], start: usize) -> Option<usize> {
    let base = indent(lines[start]);
    let mut end = start;
    for index in start + 1..lines.len() {
        let line = lines[index];
        if line.trim().is_empty() {
            continue;
        }
        if indent(line) <= base {
            return Some(end);
        }
        end = index;
    }
    Some(end)
}

fn indent(line: &str) -> usize {
    line.chars().take_while(|character| *character == ' ').count()
}

fn ts_body_end(lines: &[&str], start: usize) -> Result<Option<usize>> {
    let mut state = ScanState::Code;
    let mut depth = 0i64;
    let mut body = false;
    for index in start..lines.len() {
        let code = scan_line(lines[index], &mut state, SeedLanguage::TypeScript);
        if !body {
            if code.contains('{') || code.contains("=>") {
                body = true;
            } else if code.trim_end().ends_with(';') {
                // A declaration without a body is an overload signature; the
                // implementation is another matching line.
                return Ok(None);
            } else {
                continue;
            }
        }
        for character in code.chars() {
            match character {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        if depth < 0 {
            bail!("brace depth went negative at line {}", index + 1);
        }
        if body && depth == 0 && code.contains('}') {
            if !code
                .trim()
                .chars()
                .all(|character| matches!(character, '}' | ';' | ','))
            {
                return Ok(None);
            }
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn rust_body_end(lines: &[&str], start: usize) -> Result<Option<usize>> {
    let mut state = ScanState::Code;
    let mut depth = 0i64;
    let mut body = false;
    for index in start..lines.len() {
        let code = scan_line(lines[index], &mut state, SeedLanguage::Rust);
        if !body {
            if code.contains('{') {
                body = true;
            } else if code.trim_end().ends_with(';') {
                return Ok(None);
            } else {
                continue;
            }
        }
        for character in code.chars() {
            match character {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        if depth < 0 {
            bail!("brace depth went negative at line {}", index + 1);
        }
        if body && depth == 0 && code.contains('}') {
            if lines[index].trim() != "}" {
                return Ok(None);
            }
            return Ok(Some(index));
        }
    }
    Ok(None)
}

/// Return the line with comments and string contents removed, so braces and
/// semicolons can be counted without being fooled by literals.
fn scan_line(line: &str, state: &mut ScanState, language: SeedLanguage) -> String {
    let characters: Vec<char> = line.chars().collect();
    let mut code = String::with_capacity(line.len());
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        match *state {
            ScanState::Code => {
                if character == '/' && characters.get(index + 1) == Some(&'/') {
                    break;
                }
                if character == '/' && characters.get(index + 1) == Some(&'*') {
                    *state = ScanState::BlockComment;
                    index += 2;
                    continue;
                }
                if language == SeedLanguage::Rust && character == 'r' {
                    if let Some(hashes) = raw_string_hashes(&characters[index..]) {
                        *state = ScanState::RawText(hashes);
                        index += 2 + hashes;
                        continue;
                    }
                }
                if character == '"' {
                    *state = ScanState::Text('"');
                    index += 1;
                    continue;
                }
                if character == '`' {
                    *state = ScanState::Text('`');
                    index += 1;
                    continue;
                }
                if character == '\'' {
                    if language == SeedLanguage::Rust && is_lifetime(&characters[index..]) {
                        index += 1;
                        continue;
                    }
                    *state = ScanState::Text('\'');
                    index += 1;
                    continue;
                }
                code.push(character);
                index += 1;
            }
            ScanState::BlockComment => {
                if character == '*' && characters.get(index + 1) == Some(&'/') {
                    *state = ScanState::Code;
                    index += 2;
                    continue;
                }
                index += 1;
            }
            ScanState::Text(quote) => {
                if character == '\\' {
                    index += 2;
                    continue;
                }
                if character == quote {
                    *state = ScanState::Code;
                }
                index += 1;
            }
            ScanState::RawText(hashes) => {
                if character == '"' && characters.get(index + 1..index + 1 + hashes).is_some_and(
                    |following| following.iter().all(|next| *next == '#'),
                ) {
                    *state = ScanState::Code;
                    index += 1 + hashes;
                    continue;
                }
                index += 1;
            }
        }
    }
    code
}

/// `r"`, `r#"`, `r##"` … start a raw string; return the hash count.
fn raw_string_hashes(characters: &[char]) -> Option<usize> {
    if characters.first() != Some(&'r') {
        return None;
    }
    let mut index = 1;
    while characters.get(index) == Some(&'#') {
        index += 1;
    }
    (characters.get(index) == Some(&'"')).then_some(index - 1)
}

fn is_lifetime(characters: &[char]) -> bool {
    if characters.first() != Some(&'\'') {
        return false;
    }
    let next = characters.get(1);
    match next {
        Some(character) if character.is_ascii_alphabetic() || *character == '_' => {
            characters.get(2) != Some(&'\'')
        }
        _ => false,
    }
}

fn documentation(
    lines: &[&str],
    language: SeedLanguage,
    start: usize,
) -> Option<(usize, usize, String)> {
    match language {
        SeedLanguage::TypeScript => ts_documentation(lines, start),
        SeedLanguage::Python => python_documentation(lines, start),
        SeedLanguage::Rust => rust_documentation(lines, start),
    }
}

fn ts_documentation(lines: &[&str], start: usize) -> Option<(usize, usize, String)> {
    let mut close = None;
    let mut index = start;
    while index > 0 {
        index -= 1;
        let line = lines[index];
        if line.trim().is_empty() {
            break;
        }
        if line.trim_end().ends_with("*/") {
            close = Some(index);
            break;
        }
    }
    let close = close?;
    let mut open = close;
    while open > 0 && !lines[open].trim_start().starts_with("/**") {
        open -= 1;
    }
    if !lines[open].trim_start().starts_with("/**") {
        return None;
    }
    let text = doc_prose(&lines[open..=close], true);
    Some((open, close, text))
}

fn python_documentation(lines: &[&str], start: usize) -> Option<(usize, usize, String)> {
    // A signature may span several lines; the docstring begins after the line
    // that closes it with a colon.
    let mut signature_end = start;
    while signature_end < lines.len() {
        let code = lines[signature_end]
            .split('#')
            .next()
            .unwrap_or_default()
            .trim_end();
        if code.ends_with(':') {
            break;
        }
        signature_end += 1;
    }
    let mut index = signature_end + 1;
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }
    let line = lines.get(index)?;
    let trimmed = line.trim_start();
    for quote in ["\"\"\"", "'''"] {
        let Some(position) = trimmed.find(quote) else {
            continue;
        };
        let after = &trimmed[position + quote.len()..];
        if let Some(close) = after.find(quote) {
            return Some((index, index, after[..close].trim().to_string()));
        }
        let mut text = String::from(after.trim());
        for (offset, candidate) in lines.iter().enumerate().skip(index + 1) {
            if let Some(close) = candidate.find(quote) {
                text.push(' ');
                text.push_str(candidate[..close].trim());
                return Some((index, offset, text));
            }
            text.push(' ');
            text.push_str(candidate.trim());
        }
    }
    // A one-line string literal is a valid docstring, too.
    for quote in ['"', '\''] {
        if let Some(rest) = trimmed.strip_prefix(quote) {
            if let Some(close) = rest.find(quote) {
                return Some((index, index, rest[..close].to_string()));
            }
        }
    }
    None
}

fn rust_documentation(lines: &[&str], start: usize) -> Option<(usize, usize, String)> {
    let mut index = start;
    let mut doc_lines = Vec::new();
    while index > 0 {
        index -= 1;
        let trimmed = lines[index].trim_start();
        if trimmed.starts_with("///") {
            doc_lines.push((index, trimmed.trim_start_matches('/').trim().to_string()));
            continue;
        }
        if trimmed.starts_with("#[") || trimmed.starts_with("//") {
            continue;
        }
        break;
    }
    if doc_lines.is_empty() {
        return None;
    }
    doc_lines.reverse();
    let text = doc_lines
        .iter()
        .map(|(_, line)| line.trim_start_matches('!').trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let first = doc_lines[0].0;
    let last = doc_lines.last().map(|(index, _)| *index)?;
    Some((first, last, text))
}

/// Prose of a documentation block: markers removed, tags dropped, whitespace
/// collapsed. A block that only carries tags has no prose.
fn doc_prose(lines: &[&str], block: bool) -> String {
    let mut out = Vec::new();
    for line in lines {
        let mut text = line.trim();
        if block {
            text = text.trim_start_matches("/**").trim_start_matches('/');
            text = text.trim_end_matches("*/");
        } else {
            text = text.trim_start_matches('/');
        }
        let text = text.trim_start_matches('*').trim();
        if text.starts_with('@') {
            continue;
        }
        if text.is_empty() {
            continue;
        }
        out.push(text);
    }
    out.join(" ").split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Review checks
// ---------------------------------------------------------------------------

/// Read a reviewed gold file and compare it with fresh proposals. A reviewed
/// file that no longer matches its seeds and checkout is a review error.
pub fn check_reviewed(
    gold_root: &Path,
    repository: &str,
    metric: &str,
    proposed: &[serde_json::Value],
) -> Result<()> {
    let path = reviewed_path(gold_root, repository, metric);
    let text =
        fs::read_to_string(&path).with_context(|| format!("read reviewed gold {}", path.display()))?;
    let mut recorded = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        recorded.push(
            serde_json::from_str::<serde_json::Value>(line)
                .with_context(|| format!("parse reviewed gold {} line {}", path.display(), index + 1))?,
        );
    }
    anyhow::ensure!(
        recorded == proposed,
        "reviewed gold {} {} does not match its seeds and checkout",
        repository,
        metric
    );
    Ok(())
}

/// Fingerprint of a gold file, as the manifest records it.
pub fn fingerprint(path: &Path) -> Result<String> {
    crate::manifest::file_fingerprint(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_queries_are_derived_from_the_symbol_and_the_doc() {
        let definition = Definition {
            path: "src/a.ts".to_string(),
            symbol: "createExternalModuleSymbol".to_string(),
            start_line: 10,
            end_line: 12,
            doc_start: 8,
            doc_end: 9,
            doc_text: "Creates an external module symbol for a source file.".to_string(),
            concept: None,
            natural_language: None,
            equivalence: None,
            typo: None,
        };
        assert_eq!(concept_query(&definition), "external module symbol");
        assert_eq!(
            equivalence_query(&definition),
            "external module symbol creation"
        );
        assert_eq!(typo_query(&definition), "createExternalModuleSymblo");
        assert_eq!(
            natural_language_query(&definition),
            "Where is the code that creates an external module symbol for a source file?"
        );
        let facts = required_facts(&definition);
        assert!(facts.len() >= 2, "facts: {facts:?}");
        assert!(
            facts
                .iter()
                .any(|forms| forms.iter().any(|form| form == "source")),
            "facts: {facts:?}"
        );
        assert!(
            !facts
                .iter()
                .any(|forms| forms.iter().any(|form| form == "external")),
            "a word already in the symbol name is not distinctive: {facts:?}"
        );
    }

    #[test]
    fn s12_definition_scan_resolves_ts_python_and_rust() {
        let dir = std::env::temp_dir().join(format!("benchmark-scan-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("a.ts"),
            "/**\n * Creates a source file from text.\n */\nexport function createSourceFile(file: string): number {\n    return file.length;\n}\n",
        )
        .unwrap();
        fs::write(
            dir.join("b.py"),
            "def read_text(self):\n    \"\"\"Read the whole file as text.\"\"\"\n    return self._read()\n",
        )
        .unwrap();
        fs::write(
            dir.join("c.rs"),
            "/// Swaps two values.\n#[inline]\npub fn swap<T>(a: &mut T, b: &mut T) {\n    core::mem::swap(a, b);\n}\n",
        )
        .unwrap();
        let ts = resolve_seed(
            &dir,
            SeedLanguage::TypeScript,
            &Seed { path: "a.ts".into(), symbol: "createSourceFile".into(), concept: None, natural_language: None, equivalence: None, typo: None },
        )
        .expect("ts definition");
        assert_eq!((ts.start_line, ts.end_line), (4, 6));
        assert_eq!((ts.doc_start, ts.doc_end), (1, 3));
        let python = resolve_seed(
            &dir,
            SeedLanguage::Python,
            &Seed { path: "b.py".into(), symbol: "read_text".into(), concept: None, natural_language: None, equivalence: None, typo: None },
        )
        .expect("python definition");
        assert_eq!((python.start_line, python.end_line), (1, 3));
        assert_eq!(python.doc_text, "Read the whole file as text.");
        let rust = resolve_seed(
            &dir,
            SeedLanguage::Rust,
            &Seed { path: "c.rs".into(), symbol: "swap".into(), concept: None, natural_language: None, equivalence: None, typo: None },
        )
        .expect("rust definition");
        assert_eq!((rust.start_line, rust.end_line), (3, 5));
        assert_eq!((rust.doc_start, rust.doc_end), (1, 1));
        fs::remove_dir_all(&dir).ok();
    }
}
