# `chain-selector` human summary template

```
🔗 Chain plan emitted — `cuo/cpo/chain-selector` v{skill_version}
📄 For brief: {brief_path}
🏷️ Profile: {profile_emoji} {chain_profile}
   {if user_overrode: ↺ User overrode auto-selection: was {auto_profile}, now {chain_profile}}
   {if user_overrode: Reason: {override_reasoning}}

🔢 Chain plan ({skill_count} skills):
   {for each skill: → <skill_id>}

📊 Trace: {trace_id}  |  Audit row: {audit_row_id}
```

## Profile emoji

| Profile | Emoji |
| --- | --- |
| lean | 🪶 |
| standard | 📋 |
| full | 🏛️ |
