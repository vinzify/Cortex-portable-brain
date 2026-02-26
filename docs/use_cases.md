# Use Cases

## 1) Personal Assistant With Changing Preferences
Keep durable user preferences in one brain even when the model provider changes.
When old and new preferences conflict, RMVM conflict groups and deterministic planning avoid hidden overwrite behavior.
The assistant can still answer with verified blocks tied to proof roots.
Use suppression when a preference should stop affecting future outputs without deleting history.

```bash
cortex brain forget --subject user:local --predicate prefers_beverage --reason "suppress preference"
```

## 2) Coding Agent Safety With Taint Controls
Use attachments to scope what a coding agent can read/write and where it can sink output.
Untrusted web-derived data can be kept out of tool and policy sinks via taint rules in RMVM execution.
This reduces prompt-injection-to-tooling risk while preserving useful memory for normal conversation.
Teams can run the same agent against the same brain with explicit least-privilege grants.

```bash
cortex brain attach --agent coder --model gpt-4o --read normative.preference,project.note --write project.note --sinks narrative
```

## 3) Enterprise Audit And Incident Review
Every proxy response includes `semantic_root` and `trace_root` metadata for log correlation.
During incident review, teams can trace which memory set influenced an answer across provider changes.
Deterministic execute results and proof roots support reproducible audit trails instead of opaque prompts.
This gives compliance teams a stable control surface without coupling to one model vendor.

```bash
curl -i http://127.0.0.1:8080/v1/chat/completions -H "Authorization: Bearer ctx_demo_key" -H "Content-Type: application/json" -d "{\"model\":\"cortex-brain\",\"messages\":[{\"role\":\"user\",\"content\":\"What do I prefer?\"}]}"
```
