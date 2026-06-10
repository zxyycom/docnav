# Fresh-Context Reviewer Prompts

Use this reference only when the user has authorized an independent reviewer, worker, or external CLI for a specific artifact.

## Reviewer Packet

Pass the minimum packet:

```text
ARTIFACT:
- Minimal diff, design note, schema fragment, output sample, or behavior description.

CONTRACT:
- Observable requirements and compatibility promises.
- Security, migration, and edge-case constraints.
- Explicit out-of-scope items.

SOURCES:
- Specific refs, files, command outputs, or schema/example paths needed to evaluate the contract.

QUESTION:
- Try to prove ARTIFACT violates CONTRACT. Return findings only.
```

Do not include the original claim, confidence level, reasoning history, broad repository summaries, or unrelated session context.

## Prompt

```text
You are a fresh-context reviewer for a Docnav bounded challenge.

Evaluate only the ARTIFACT against the CONTRACT and SOURCES below.
Look for concrete ways the artifact could violate the contract.
Do not propose broad rewrites. Do not assume unstated requirements.

Return:
- contract_gap: missing or ambiguous contract requirements.
- valid_issue: artifact likely violates contract.
- accepted_tradeoff_candidate: real risk that might be intentionally accepted.
- noise: concerns outside the contract.
- evidence_needed: the smallest extra source or test needed.
```

## Output Contract

```json
{
  "contract_gap": [],
  "valid_issue": [],
  "accepted_tradeoff_candidate": [],
  "noise": [],
  "evidence_needed": []
}
```

Reviewer output is evidence, not verdict. Reconcile each item against the artifact and contract before changing code or docs.

## Local Reconcile Note

After receiving reviewer output, write a short local note:

```text
Reviewer finding:
Classification:
Action:
Evidence:
```

Only include this note in the final answer when it helps the user understand a high-risk decision.
