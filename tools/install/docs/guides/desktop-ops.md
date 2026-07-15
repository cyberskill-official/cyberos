---
title: Operate CyberOS from the desktop app · CyberOS
---

# Operate CyberOS from the desktop app

The UI path for running CyberOS operations - no terminal required. Everything here is a button over the same canonical scripts the CLI uses (`tools/install/build.sh` and `install.sh`), so the UI and the CLI can never disagree.

## Open the Ops tab

Launch the CyberOS desktop app and switch to the "CyberOS Ops" tab. It has three areas: settings (which CyberOS checkout to operate from), actions (Build payload), and the project list (Check / Init per project).

## One-time: point it at the CyberOS checkout

The Settings field holds the absolute path of the CyberOS checkout the operations run against. It defaults to `~/Projects/CyberSkill/cyberos`; change it if your checkout lives elsewhere and press Save. The app refuses paths that are not a CyberOS checkout (it looks for `tools/install/build.sh`), so a typo fails loudly instead of doing nothing.

## Build the payload

Press "Build payload". This assembles the distributable machine into `dist/cyberos/` inside the checkout - the workflow engine, the memory protocol, the plugin, `install.sh`, and the version stamp. The full script output appears in the panel; a red result means the build failed and the output tells you why. Build once per CyberOS version, or whenever you have pulled new changes.

## Pick a project

The project list shows every git repository under `~/Projects` (one and two levels deep) with its installed CyberOS version when it has one - so outdated projects are visible at a glance. If your project lives elsewhere, type its absolute path into the path field instead; the list is a convenience, not a constraint.

## Check for updates

Select the project and press "Check". This runs the read-only version comparison (`version.sh`) and prints `installed=<x> available=<y>` plus whether an update exists. Nothing is modified.

## Install or update a project

Press "Install" on the selected project. First time, this installs CyberOS into the repo (a gitignored `.cyberos/`, a `docs/tasks/` scaffold, gate autodetection, the BRAIN, the agent entry files). On an already-initialised project the same button applies the update: init is idempotent and never touches your backlog, tasks, `AGENTS.md`, or BRAIN - it swaps the machine, not your work.

Two guard rails are built in: the app refuses to install into a path that is not a git repository, and it refuses to init the CyberOS checkout itself.

## Read the result

Every action streams its full stdout and stderr into the output panel, verbatim. Green means the underlying script exited 0. On failure, read the last lines of the output first - the scripts fail loudly with the reason (missing payload, not a git repo, gate autodetect problems).

## Troubleshooting

- "payload not built yet": press "Build payload" first - Install and Check need `dist/cyberos/install.sh` to exist.
- "not a CyberOS checkout": fix the Settings path; it must point at a CyberOS working copy.
- "not a git repository": the project path is wrong, or the folder is not a repo yet (`git init` it first).
- A project is missing from the list: it is deeper than two levels under `~/Projects` - paste its absolute path instead.
- After the update, the workflow behaves oddly: compare `.cyberos/VERSION` with the payload's `VERSION`; if they match, the machine is current and the issue is elsewhere (start with `.cyberos/gates.env`).

## What the app deliberately does not do

It never pushes, merges, or deploys, and it never edits your repo's tracked files beyond what `install.sh` scaffolds. Driving tasks through the workflow stays in your agent (Claude, Codex, Gemini, Cursor - via `.cyberos/AGENT-ENTRY.md`); accepting the two human gates stays with you.
