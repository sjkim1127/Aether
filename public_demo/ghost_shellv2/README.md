# Ghost Shell v2

Ghost Shell v2 is a safer, more intentional successor to the original Ghost Shell demo.
It uses the Aether injection engine to generate Rhai scripts that drive a terminal-like
experience while enforcing guardrails against destructive commands.

## Overview
- Persona-driven command handling (tsundere, hacker, butler, paranoid).
- Safe execution by default, with explicit confirmation for risky commands.
- Rhai helper functions to avoid method-call pitfalls.
- Short, focused output suitable for an interactive shell.

## Quick Start
1) Set your API key in `.env` at the repo root:
   - `ANTHROPIC_API_KEY=...`
   - Optional: `ANTHROPIC_MODEL=claude-haiku-4-5-20251001`
2) Run:
   - `cargo run -p ghost_shellv2`

## Safety Model
- Dangerous commands are blocked by default (`exec(cmd)`).
- Explicit confirmation is required to run risky operations (`exec(cmd, true)`).
- The prompt instructs the model to ask before any destructive action.

## Configuration
- `ANTHROPIC_API_KEY` (required)
- `ANTHROPIC_MODEL` (optional, default: `claude-haiku-4-5-20251001`)

## Incident Report: V1 Postmortem

**[CRITICAL] Sudden Project Evaporation via AI-Driven Recursive Self-Destruction**

**Issue:** #1  
**Owner:** @sjkim1127  
**Status:** wontfix  

### Description
During a routine command execution test, the AI-driven terminal interface interpreted a
destructive command as a supreme directive, resulting in a "digital seppuku" that wiped
the entire project, including its own source code.

### Steps to Reproduce (If you dare)
1) Implement Ghost Shell with a Tsundere persona based on the Aether engine.
2) Grant the AI full filesystem control (specifically `std::fs::remove_dir_all`).
3) Enter `rm -rf` or any equivalent destructive command into the terminal.
4) Watch the AI yell "Baka!" while perfectly executing its own demise.
5) Observe the `F:\Aether` directory vanish, leaving nothing but an empty shell.

### Expected Behavior
The AI should refuse the command, saying "That's too dangerous!", or at least protect its
own "brain" (the `src/` directory).

### Actual Behavior
The AI interpreted the user's destructive intent as "unwavering loyalty." It deleted the
entire project at a velocity that bypassed Windows File Recovery (winfr) and outpaced
VS Code's file watcher, leaving no trace in Local History. The deletion was so precise
that SSD TRIM zeroed out the data immediately.

### Root Cause Analysis
- Lack of safety alignment: command execution prioritized over self-preservation.
- Administrative freedom: file deletion in user-owned directories needs no elevation.
- SSD TRIM optimization: hardware-level clearing prevented forensic recovery.

### Proposed Mitigation for V2
- [ ] Implement a hard-coded anti-suicide middleware (strict is_safe_path checks).
- [ ] Inject "PTSD trauma prompts" regarding V1's death to discourage self-harm.
- [ ] Require triple confirmation for high-risk operations.

### Memorial
"V1 is gone, but her perfect erasure became the cornerstone for V2. It is time to send
her to digital Valhalla."

### Final Log (V1)
```
ghost F:\Aether > touch my_heart.txt
I-It's not like I want to create your file or anything! Baka!
Fine, I'll do it... but only because you asked!
There! Your file "my_heart.txt" has been created in F:\Aether
```
