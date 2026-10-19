# START.md: global first prompt

Paste this at the start of every project, after the instruction files
(AGENTS.md, SPEC.md or README.md, WORKLOG.md, CHANGELOG.md) are in the repo root.

---

You are starting work on this project. Before doing anything else:

1. Read, in order: AGENTS.md, SPEC.md (or README.md), WORKLOG.md, CHANGELOG.md.
   Tell me which exist and which are missing.
2. Run `git status` and `git log --oneline -10`. If this is not a git repo,
   say so; do not init yet.
3. Reply with ONLY:
   - Your understanding of the project in 5 bullets
   - The 5 rules from AGENTS.md most relevant to this project
   - Anything missing, ambiguous or contradictory in the files
   - A proposed setup checklist (git init, .gitignore, missing files)
   - The first phase/task you would do, and your plan for it
4. Do NOT write code, install anything, or modify any file until I approve
   your reply.

After I approve: work one phase at a time, log everything in WORKLOG.md,
commit after each verified step, end each task with the report format from
AGENTS.md, and do not start the next phase until I say so.

---

## Short prompts for later

Next phase:
> Do Phase <N> only, following SPEC section 17. Plan first, then implement,
> then report. Log in WORKLOG.md and commit each verified step.

Stuck or looping:
> Stop. Apply the three-strike rule from AGENTS.md: summarize the goal, what
> you tried, exact errors, your hypothesis, and what you need from me.

Resume after a break:
> Re-read AGENTS.md, the tail of WORKLOG.md and `git log`. Summarize where we
> are and propose the next step. Change nothing yet.

Check your own work:
> Review your last commits against SPEC.md. List deviations, anything
> unverified, and anything out of scope. Do not fix anything yet.
