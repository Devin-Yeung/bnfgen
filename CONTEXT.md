# Bnfgen

BNF-extended grammars for random string generation, with semantic analysis suitable for tools (CLI, future LSP).

## Language

### Grammar pipeline

**Raw Grammar**:
The parsed grammar document before validation. Retains rules, alternatives, symbols, and source spans.
_Avoid_: AST (alone), parse tree, unvalidated grammar

**Checked Grammar**:
A validated rule table indexed by non-terminal, safe to generate from. Does not retain source spans and does not perform reduction.
_Avoid_: validated AST, compiled grammar, ready grammar

**Checked Production**:
A validated production belonging to a Checked Grammar, expressed in checked symbols.
_Avoid_: WeightedProduction (when referring to the checked stage)

**Grammar Graph**:
A dependency graph over grammar rules used for reachability and trap-loop analysis.
_Avoid_: call graph, rule index

### Symbols

**Symbol Kind**:
A symbol as it appears in the Raw Grammar: terminal, non-terminal, or regex literal.
_Avoid_: Checked Symbol Kind, token

**Checked Symbol Kind**:
A symbol as it appears in the Checked Grammar: terminal, non-terminal, or regex engine.
_Avoid_: Symbol Kind (when referring to the checked stage)

**Non-Terminal**:
A named rule reference, optionally typed (e.g. `<E: "int">`).
_Avoid_: rule name, identifier, variable

**Terminal**:
A literal string that appears in generated output.
_Avoid_: string, constant, token

**Regex Lit**:
A regex pattern as written in the grammar source (pattern text + span). Validated at parse; does not produce strings.
_Avoid_: Regex, pattern, Regex Engine

**Regex Engine**:
A compiled regex used during generation to produce matching strings.
_Avoid_: Regex Lit, Hir, compiled pattern

### Alternatives & generation

**Alternative**:
One weighted branch of a production, with an optional invoke limit. Pure data in the Raw Grammar.
_Avoid_: branch, choice, candidate (except during expansion)

**Invoke Limit**:
A min/max constraint on how many times an Alternative may be selected during a generation run.
_Avoid_: repeat count, quantifier, range (alone)

**Generator**:
The entry point that expands a Checked Grammar from a start non-terminal into strings (or a generation tree).
_Avoid_: Reducer, expander, sampler

**Expansion**:
The generation-side process of choosing alternatives and replacing symbols until only terminals remain.
_Avoid_: reduce (as a domain noun), evaluation, interpretation
