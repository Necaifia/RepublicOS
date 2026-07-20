# RFC 004: Knowledge Graph Schema and Query Language

**Status**: Draft
**Date**: Genesis
**Author**: Pantheon Founding Scientific Council

---

## Summary

Pantheon's semantic memory is a distributed knowledge graph. This RFC specifies the node types, edge types, and query language.

## Motivation

Without a standard schema:
- Different agents will model the same knowledge differently
- Cross-agent querying becomes impossible
- Knowledge integration (merging) becomes ad-hoc

## Node Types

### Code Entities

| Node Type | Properties | Example |
|-----------|------------|---------|
| CodeRepository | name, url, language, owner | pantheon/kernel |
| CodeModule | name, path, language | payment/src/processors |
| CodeFile | path, language, lines, hash | payment/src/processors/checkout.rs |
| CodeFunction | name, signature, complexity, lines | process_payment |
| CodeClass | name, methods, fields, parent | PaymentProcessor |
| CodeInterface | name, methods, implementors | PaymentGateway |
| CodeEnum | name, variants | PaymentStatus |
| CodeTest | name, type (unit/integration/e2e) | test_process_payment |
| CodeBug | title, severity, status, fix_version | BUG-4382 |

### Domain Concepts

| Node Type | Properties | Example |
|-----------|------------|---------|
| DomainConcept | name, definition, references | Payment |
| BusinessRule | description, source, scope | max_refund_amount = 5000 |
| ArchitectureDecision | title, context, decision, consequences | Use event sourcing for payments |
| DesignPattern | name, description, used_in | Saga pattern |
| Technology | name, version, purpose | PostgreSQL 15 |

### Agent Entities

| Node Type | Properties | Example |
|-----------|------------|---------|
| AgentType | role, capabilities, model | Developer Agent |
| AgentInstance | id, status, parent, created_at | agent://dev-4382 |
| Decision | agent, timestamp, context, outcome | Decided to use Stripe API |
| Task | id, type, status, assignee, result | TASK-8821 |
| Conversation | participants, messages, resolution | Debug session for BUG-12 |

## Edge Types

| Edge Type | Source → Target | Meaning |
|-----------|----------------|---------|
| depends_on | CodeModule → CodeModule | A depends on B |
| implements | CodeFunction → CodeInterface | Function implements interface |
| extends | CodeClass → CodeClass | A extends B |
| calls | CodeFunction → CodeFunction | A calls B |
| defines | CodeFile → CodeFunction | File contains function |
| references | Any → Any | A references B (general) |
| causes | Bug → CodeFunction | Bug caused by function |
| fixes | Commit → Bug | Commit fixes bug |
| decides | Decision → Any | Decision affects entity |
| relates_to | Any → Any | A is related to B |
| part_of | CodeFunction → CodeModule | Function belongs to module |
| satisfies | CodeTest → CodeFunction | Test tests function |
| documents | Doc → CodeFunction | Document describes function |

## Query Language

### Pattern Matching

```
// Find all functions called by processPayment
MATCH (f:CodeFunction {name: "processPayment"}) -[:calls]-> (callee)
RETURN callee.name, callee.complexity

// Find all tests for payment module
MATCH (m:CodeModule {name: "payment"})<-[:part_of]-(f:CodeFunction)
<-[:satisfies]-(t:CodeTest)
RETURN t.name, f.name

// Find the root cause of a bug
MATCH (b:Bug {id: "BUG-4382"}) -[:causes]-> (bad_func)
<-[:depends_on*]-(all_affected)
RETURN bad_func, all_affected
```

### Semantic Search

```
// Find similar functions
SIMILARITY("refundPayment", top_k=5, domain="payment")

// Find entities related to a concept
RELATED("PCI compliance", max_depth=3)
```

### Temporal Queries

```
// What did we know about payment module at version 1.0?
SNAPSHOT("payment", version="1.0")

// How has the architecture decision changed over time?
HISTORY("Use event sourcing for payments")
```

## Indexing

- Text index: Full-text search on all string properties
- Vector index: Embedding-based similarity search on selected properties
- Graph index: Adjacency lists for fast traversal
- Temporal index: Time-based queries

## Distribution

The knowledge graph is sharded by domain. Shard boundaries are managed by the Memory Service. Cross-shard queries use a scatter-gather pattern.
