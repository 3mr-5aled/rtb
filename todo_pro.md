أيوه. الـ backlog الحالي جيد كبداية، لكن فيه شوية gaps مهمة لو هدف \*\*RTB\*\* يبقى فعلًا Developer Project Operations tool وليس مجرد project launcher.



أنا هعدّله بالشكل ده:



\# RTB — رتّب

\*\*Repository \& Tooling Base\*\*



\## Main



\- \[ ] Change project name to \*\*RTB (رتّب)\*\*

\- \[ ] Update all branding, CLI commands, README, TUI title, installer, and config references

\- \[ ] Rename CLI from `dev` → `rtb`

\- \[ ] Rename TUI command from `devtui` → `rtbtui` or decide whether TUI should remain an internal implementation detail

\- \[ ] Define RTB's core identity and command philosophy



\---



\# Improvements



\### UX / TUI



\- \[ ] More clean and clear start page

\- \[ ] Show only actionable/high-value information on Dashboard

\- \[ ] Project quick actions from Dashboard

\- \[ ] Consistent keyboard shortcuts across all screens

\- \[ ] Clear visual indication of selected/active project

\- \[ ] Confirmation prompts for destructive operations

\- \[ ] Non-blocking progress indicators for long-running operations

\- \[ ] Better error messages with actionable recovery suggestions

\- \[ ] Global command/help overlay

\- \[ ] `q` / `Esc` behavior standardized across views



\### CLI



\- \[ ] Consistent command output format

\- \[ ] `--help` for every command

\- \[ ] `--version`

\- \[ ] `--verbose` / debug mode

\- \[ ] Proper exit codes

\- \[ ] Colored output with graceful fallback when colors aren't supported

\- \[ ] Shell completion

\- \[ ] Better argument validation



\---



\# Features



\### Project Intelligence



\- \[ ] Detect project type automatically

\- \[ ] Detect frameworks and libraries

\- \[ ] Detect package managers

&#x20; - \[ ] npm

&#x20; - \[ ] pnpm

&#x20; - \[ ] yarn

&#x20; - \[ ] bun

&#x20; - \[ ] pip

&#x20; - \[ ] poetry

&#x20; - \[ ] uv

&#x20; - \[ ] cargo

\- \[ ] Python and other runtime environment detection

\- \[ ] Node.js version detection

\- \[ ] Python version / virtual environment detection

\- \[ ] Rust toolchain detection

\- \[ ] Java / Go / .NET detection

\- \[ ] Docker detection

\- \[ ] Docker Compose detection

\- \[ ] Environment/config detection (`.env`, `.env.example`, etc.)

\- \[ ] Git repository detection

\- \[ ] Git branch/status detection



\### Git



\- \[ ] Git repository health overview

\- \[ ] Details on git commit

\- \[ ] Small commit window for easier commits

\- \[ ] Stage / unstage files

\- \[ ] Commit message generation/editing

\- \[ ] Branch management

\- \[ ] Ahead/behind remote status

\- \[ ] Uncommitted changes indicator

\- \[ ] Untracked files indicator

\- \[ ] Last commit information

\- \[ ] Optional push after commit



\### Project Operations



\- \[ ] Open project in VS Code

\- \[ ] Open project in default file manager

\- \[ ] Open project terminal

\- \[ ] Open project in browser

\- \[ ] Open in Agent CLI

\- \[ ] Run project

\- \[ ] Install dependencies

\- \[ ] Update dependencies

\- \[ ] Build project

\- \[ ] Test project

\- \[ ] Project-specific scripts discovery

\- \[ ] Run arbitrary `package.json` / project scripts



\### Documentation



\- \[ ] Open Markdown with rendered/styled view

\- \[ ] Markdown preview

\- \[ ] README detection

\- \[ ] PROJECT.md integration

\- \[ ] Project metadata viewer



\---



\# Project Lifecycle



ده جزء أساسي في RTB، وأعتقد يستحق section مستقل:



\- \[ ] Create

\- \[ ] Activate

\- \[ ] Run

\- \[ ] Pause

\- \[ ] Resume

\- \[ ] Maintain

\- \[ ] Deploy

\- \[ ] Archive

\- \[ ] Restore

\- \[ ] Delete



والـ TUI يعرض lifecycle state بوضوح:



```text

ACTIVE

PAUSED

DEPLOYED

ARCHIVED

BROKEN

UNKNOWN

```



\---



\# Compatibility



\### Filesystem



\- \[ ] Compatibility with empty folders

\- \[ ] Compatibility with custom folder structures

\- \[ ] Don't assume `D:\\02-Projects\\...`

\- \[ ] Configurable project roots

\- \[ ] Multiple project roots

\- \[ ] Projects outside the configured root

\- \[ ] Graceful handling of missing directories

\- \[ ] Symlink support



\### Operating Systems



\- \[ ] Windows

\- \[ ] Linux

\- \[ ] macOS



\### Shells



\- \[ ] PowerShell

\- \[ ] Bash

\- \[ ] Zsh

\- \[ ] Fish



\*\*مهم:\*\* افصل بين \*OS compatibility\* و\*shell compatibility\*. دول مش نفس المشكلة.



\---



\# Configuration



دي ناقصة من الـ backlog الحالي:



\- \[ ] Move environment-specific paths completely into configuration

\- \[ ] User-level configuration

\- \[ ] Project-level configuration

\- \[ ] Configuration validation

\- \[ ] Configuration migration/versioning

\- \[ ] Config reset command

\- \[ ] Config initialization command



مثلاً:



```bash

rtb init

```



يعمل setup للبيئة بدل ما installer يفترض structure محدد.



\---



\# Developer Experience



ده أهم section بالنسبة لي:



\- \[ ] Fast startup time

\- \[ ] Lazy-load expensive operations

\- \[ ] Don't scan every project unless necessary

\- \[ ] Cache project metadata

\- \[ ] Background health checks where possible

\- \[ ] Clear loading states

\- \[ ] Never freeze the TUI during filesystem/git operations

\- \[ ] Graceful Ctrl+C handling

\- \[ ] Graceful handling of broken projects

\- \[ ] Handle projects with missing dependencies

\- \[ ] Handle projects with corrupted Git repositories

\- \[ ] Handle permission errors

\- \[ ] Handle missing runtimes/tools



\---



\# Safety



بما أن RTB هيعمل operations زي archive / clean / delete:



\- \[ ] Confirmation before destructive operations

\- \[ ] `--force` only where appropriate

\- \[ ] Dry-run mode

\- \[ ] Show exactly what will be changed

\- \[ ] Never delete outside configured project scope accidentally

\- \[ ] Protect `.git`

\- \[ ] Protect configuration files

\- \[ ] Dependency cleanup should have a preview mode



مثلاً:



```bash

rtb clean --dry-run

```



ده مهم جدًا.



\---



\# Testing



أنت كتبت:



> test the program before finish



أنا أحولها إلى acceptance criteria واضحة:



\- \[ ] Unit tests for core logic

\- \[ ] CLI command tests

\- \[ ] Configuration tests

\- \[ ] Filesystem edge-case tests

\- \[ ] Git integration tests

\- \[ ] TUI interaction tests where practical

\- \[ ] Test empty folder

\- \[ ] Test custom folder structure

\- \[ ] Test missing Git

\- \[ ] Test missing runtime

\- \[ ] Test permission errors

\- \[ ] Test corrupted/malformed config

\- \[ ] Test Windows paths

\- \[ ] Test Linux paths

\- \[ ] Run full test suite before release

\- \[ ] Manual smoke test before release



\---



\# Documentation



\- \[ ] Rewrite README around RTB instead of implementation details

\- \[ ] Installation guide

\- \[ ] CLI reference

\- \[ ] TUI keyboard shortcuts

\- \[ ] Configuration reference

\- \[ ] Project structure requirements

\- \[ ] Supported runtimes

\- \[ ] Supported OS

\- \[ ] Troubleshooting

\- \[ ] Architecture documentation

\- \[ ] Contributing guide



\---



\# Future / Experimental



أخلي الحاجات دي \*\*out of scope for v1\*\* حاليًا:



\- \[ ] AI project analysis

\- \[ ] AI-generated commit messages

\- \[ ] AI project summaries

\- \[ ] Multi-agent workflow integration

\- \[ ] Remote project management

\- \[ ] Cloud deployment integrations

\- \[ ] Plugin system

\- \[ ] Project templates marketplace



دي ممكن تخلي RTB يتحول لاحقًا من \*\*project manager\*\* إلى \*\*developer workspace orchestrator\*\*.



\---



\## وأهم تعديل في الـ roadmap



أنا أقترح تقسيمه بدل قائمة features واحدة:



```text

RTB

│

├── Core

│   ├── Project discovery

│   ├── Project lifecycle

│   ├── Configuration

│   └── Filesystem abstraction

│

├── Developer Experience

│   ├── CLI

│   ├── TUI

│   ├── Navigation

│   └── Actions

│

├── Intelligence

│   ├── Runtime detection

│   ├── Framework detection

│   ├── Docker detection

│   └── Git detection

│

├── Operations

│   ├── Run

│   ├── Build

│   ├── Test

│   ├── Deploy

│   ├── Archive

│   └── Maintenance

│

└── Integrations

&#x20;   ├── VS Code

&#x20;   ├── Agent CLI

&#x20;   └── Git

```

