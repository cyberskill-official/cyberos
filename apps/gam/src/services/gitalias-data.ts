/**
 * GitAlias Library — Static Data
 *
 * Raw alias data from https://github.com/GitAlias/gitalias
 * Statically embedded — no runtime fetch needed.
 *
 * This file is intentionally large; it's pure data.
 * The query API lives in `gitalias-library.ts`.
 */

export interface I_LibraryAlias {
    name: string;
    command: string;
    category: string;
    description: string;
}

// ── Raw alias data: [name, command, category, description] ──────
const RAW: [string, string, string, string][] = [
    // Short aliases
    ['a', 'add', 'short', 'add'],
    ['b', 'branch', 'short', 'branch'],
    ['c', 'commit', 'short', 'commit'],
    ['d', 'diff', 'short', 'diff'],
    ['f', 'fetch', 'short', 'fetch'],
    ['g', 'grep', 'short', 'grep'],
    ['l', 'log', 'short', 'log'],
    ['m', 'merge', 'short', 'merge'],
    ['o', 'checkout', 'short', 'checkout'],
    ['p', 'pull', 'short', 'pull'],
    ['s', 'status', 'short', 'status'],
    ['w', 'whatchanged', 'short', 'whatchanged'],

    // Add aliases
    ['aa', 'add --all', 'add', 'add all'],
    ['ap', 'add --patch', 'add', 'add by patch'],
    ['au', 'add --update', 'add', 'add just the files that are updated'],

    // Branch aliases
    ['bm', 'branch --merged', 'branch', 'branch and only list branches whose tips are reachable from the specified commit'],
    ['bnm', 'branch --no-merged', 'branch', 'branch and only list branches whose tips are not reachable from the specified commit'],
    ['bed', 'branch --edit-description', 'branch', 'branch with edit description'],
    ['bsd', '"!f(){ branch=\\"${1:-$(git current-branch)}\\"; git config \\"branch.$branch.description\\"; };f"', 'branch', 'branch with show description'],
    ['bv', 'branch --verbose', 'branch', 'branch verbose'],
    ['bvv', 'branch --verbose --verbose', 'branch', 'branch verbose verbose'],

    // Commit aliases
    ['ca', 'commit --amend', 'commit', 'commit - amend the tip of the current branch'],
    ['cam', 'commit --amend --message', 'commit', 'commit - amend the tip and edit the message'],
    ['cane', 'commit --amend --no-edit', 'commit', 'commit - amend the tip, do not edit the message'],
    ['caa', 'commit --amend --all', 'commit', 'commit - amend the tip, staging all files'],
    ['caam', 'commit --amend --all --message', 'commit', 'commit - amend the tip, staging files, edit message'],
    ['caane', 'commit --amend --all --no-edit', 'commit', 'commit - amend the tip, staging files, no edit'],
    ['ci', 'commit --interactive', 'commit', 'commit interactive'],
    ['cm', 'commit --message', 'commit', 'commit with a message'],

    // Checkout aliases
    ['co', 'checkout', 'checkout', 'checkout - update working tree to match a branch or paths'],
    ['cong', 'checkout --no-guess', 'checkout', 'checkout with no guess'],
    ['cob', 'checkout -b', 'checkout', 'create a new branch and switch to it'],

    // Cherry-pick aliases
    ['cp', 'cherry-pick', 'cherry-pick', 'cherry-pick - apply changes from existing commits'],
    ['cpa', 'cherry-pick --abort', 'cherry-pick', 'cherry-pick abort'],
    ['cpc', 'cherry-pick --continue', 'cherry-pick', 'cherry-pick continue'],
    ['cpn', 'cherry-pick --no-commit', 'cherry-pick', 'cherry-pick without making a commit'],
    ['cpnx', 'cherry-pick --no-commit -x', 'cherry-pick', 'cherry-pick without commit, append cherry picked from line'],

    // Diff aliases
    ['dc', 'diff --cached', 'diff', 'show changes not yet staged'],
    ['ds', 'diff --staged', 'diff', 'show changes about to be committed'],
    ['dw', 'diff --word-diff', 'diff', 'show changes by word, not line'],
    ['dd', 'diff-deep', 'diff', 'show changes with preferred options'],

    // Fetch aliases
    ['fa', 'fetch --all', 'fetch', 'fetch all remotes'],
    ['fav', 'fetch --all --verbose', 'fetch', 'fetch all remotes with verbose output'],
    ['fap', 'fetch --all --prune', 'fetch', 'fetch all remotes and delete stale references'],

    // Grep aliases
    ['gn', 'grep --line-number', 'grep', 'grep with line number'],
    ['gg', 'grep-group', 'grep', 'search with preferred options'],

    // Log aliases
    ['lg', 'log --graph', 'log', 'log with graphical representation'],
    ['lo', 'log --oneline', 'log', 'log with one line per item'],
    ['lor', 'log --oneline --reverse', 'log', 'log one line per item, reverse order'],
    ['lp', 'log --patch', 'log', 'log with patch generation'],
    ['lfp', 'log --first-parent', 'log', 'log with first parent'],
    ['lto', 'log --topo-order', 'log', 'log in topological order'],
    ['ll', 'log-list', 'log', 'log list with preferred information'],
    ['lll', 'log-list-long', 'log', 'log list with long information'],

    // Ls-files aliases
    ['ls', 'ls-files', 'ls-files', 'show information about files in index and working tree'],
    ['lsd', 'ls-files --debug', 'ls-files', 'list files with debug information'],
    ['lsfn', 'ls-files --full-name', 'ls-files', 'list files with full name'],
    ['lsio', 'ls-files --ignored --others --exclude-standard', 'ls-files', 'list files that git has ignored'],

    // Merge aliases
    ['ma', 'merge --abort', 'merge', 'merge abort'],
    ['mc', 'merge --continue', 'merge', 'merge continue'],
    ['mncnf', 'merge --no-commit --no-ff', 'merge', 'merge without autocommit and with a commit even if fast-forward'],

    // Pull aliases
    ['pf', 'pull --ff-only', 'pull', 'pull if fast-forward possible, otherwise fail'],
    ['pr', 'pull --rebase', 'pull', 'pull with rebase for cleaner history'],
    ['prp', 'pull --rebase=preserve', 'pull', 'pull with rebase preserve of merge commits'],

    // Rebase aliases
    ['rb', 'rebase', 'rebase', 'rebase - forward-port local commits'],
    ['rba', 'rebase --abort', 'rebase', 'rebase abort'],
    ['rbc', 'rebase --continue', 'rebase', 'rebase continue after resolving conflict'],
    ['rbs', 'rebase --skip', 'rebase', 'rebase skip current patch'],
    ['rbi', 'rebase --interactive', 'rebase', 'rebase interactive with prompts'],
    ['rbiu', 'rebase --interactive @{upstream}', 'rebase', 'rebase interactive on unpushed commits'],
    ['fixup', '"!f() { TARGET=\\"$(git rev-parse \\"$1\\")\\"; git commit --fixup=\\"$TARGET\\" && GIT_EDITOR=true git rebase --interactive --autosquash \\"$TARGET\\"~; }; f"', 'rebase', 'fixup - amend an older commit'],

    // Reflog aliases
    ['rl', 'reflog', 'reflog', 'reference log'],

    // Remote aliases
    ['rr', 'remote', 'remote', 'remote - manage tracked repositories'],
    ['rrs', 'remote show', 'remote', 'remote show'],
    ['rru', 'remote update', 'remote', 'remote update'],
    ['rrp', 'remote prune', 'remote', 'remote prune stale branches'],

    // Revert aliases
    ['rv', 'revert', 'revert', 'revert - undo changes from existing commits'],
    ['rvnc', 'revert --no-commit', 'revert', 'revert without autocommit'],

    // Show-branch aliases
    ['sb', 'show-branch', 'show-branch', 'print a list of branches and their commits'],

    // Submodule aliases
    ['sm', 'submodule', 'submodule', 'submodule'],
    ['smi', 'submodule init', 'submodule', 'submodule init'],
    ['sma', 'submodule add', 'submodule', 'submodule add'],
    ['sms', 'submodule sync', 'submodule', 'submodule sync'],
    ['smu', 'submodule update', 'submodule', 'submodule update'],
    ['smui', 'submodule update --init', 'submodule', 'submodule update with initialize'],
    ['smuir', 'submodule update --init --recursive', 'submodule', 'submodule update with init and recursive'],

    // Status aliases
    ['ss', 'status --short', 'status', 'status with short format'],
    ['ssb', 'status --short --branch', 'status', 'status short with branch and tracking info'],

    // Alias management
    ['alias', '"!f(){ echo \\"Git Alias is project that has a collection of git alias commands.\\"; echo \\"Free open source repository <https://github.com/gitalias/gitalias>.\\"; echo \\"\\"; echo \\"To see your existing git aliases:\\"; echo \\"    git aliases\\"; };f"', 'alias-management', 'git alias help'],
    ['add-alias', '"!f() { if [ $# != 3 ]; then echo \\"Usage: git add-alias ( --local | --global ) <alias> <command>\\"; return 2; fi; git config \\"$1\\" alias.\\"$2\\" \\"$3\\"; }; f"', 'alias-management', 'create a new git alias'],
    ['move-alias', '"!f() { if [ $# != 3 ]; then echo \\"Usage: git move-alias ( --local | --global ) <alias existing name> <new alias name>\\"; return 2; fi; git config \\"$1\\" alias.\\"$3\\" \\"$(git config \\"$1\\" --get alias.\\"$2\\")\\" && git config \\"$1\\" --unset alias.\\"$2\\"; }; f"', 'alias-management', 'rename an existing git alias'],

    // Diff-* aliases
    ['diff-all', '"!for name in $(git diff --name-only \\"$1\\"); do git difftool \\"$1\\" \\"$name\\" & done"', 'diff', 'diff all files'],
    ['diff-changes', 'diff --name-status -r', 'diff', 'diff changes'],
    ['diff-stat', 'diff --stat --ignore-space-change -r', 'diff', 'diff stat'],
    ['diff-staged', 'diff --cached', 'diff', 'diff staged files'],
    ['diff-deep', 'diff --check --dirstat --find-copies --find-renames --histogram --color', 'diff', 'diff with thorough options'],

    // Grep-* aliases
    ['grep-all', '"!f() { git rev-list --all | xargs git grep \\"$@\\"; }; f"', 'grep', 'find text in any commit ever'],
    ['grep-group', 'grep --break --heading --line-number --color', 'grep', 'find text with grouped output'],
    ['grep-ack', '"-c color.grep.linenumber=\\"bold yellow\\" -c color.grep.filename=\\"bold green\\" -c color.grep.match=\\"reverse yellow\\" grep --break --heading --line-number"', 'grep', 'find text with ack-like formatting'],

    // Init aliases
    ['init-empty', '"!f() { git init && git commit --allow-empty --allow-empty-message --message \\"\\"; }; f"', 'init', 'init repo with empty rebaseable commit'],

    // Merge-span aliases
    ['merge-span', '"!f() { echo \\"$(git log -1 \\"$2\\" --merges --pretty=format:%P | cut -d\' \' -f1)$1$(git log -1 \\"$2\\" --merges --pretty=format:%P | cut -d\' \' -f2)\\"; }; f"', 'merge', 'find span of commits in a merge'],
    ['merge-span-log', '"!git log \\"$(git merge-span .. \\"$1\\")\\"', 'merge', 'find commits introduced by a merge'],
    ['merge-span-diff', '"!git diff \\"$(git merge-span ... \\"$1\\")\\"', 'merge', 'show changes introduced by a merge'],
    ['merge-span-difftool', '"!git difftool \\"$(git merge-span ... \\"$1\\")\\"', 'merge', 'show merge changes in difftool'],

    // Misc aliases
    ['last-tag', 'describe --tags --abbrev=0', 'tag', 'last tag in current branch'],
    ['last-tagged', '"!git describe --tags \\"$(git rev-list --tags --max-count=1)\\"', 'tag', 'last annotated tag in all branches'],
    ['heads', '"!git log origin/main.. --format=\'%Cred%h%Creset;%C(yellow)%an%Creset;%H;%Cblue%f%Creset\' | git name-rev --stdin --always --name-only | column -t -s\';\'"', 'log', 'show heads'],
    ['rebase-branch', '"!f() { git rebase --interactive \\"$(git merge-base \\"$(git default-branch)\\") HEAD)\\"; }; f"', 'rebase', 'interactively rebase all commits on current branch'],
    ['orphans', 'fsck --full', 'maintenance', 'find all unreferenced objects (orphans)'],
    ['rev-list-all-objects-by-size', '"!git rev-list --all --objects | awk \'{print $1}\'| git cat-file --batch-check | grep -F blob | sort -k3nr"', 'maintenance', 'list all blobs by size'],

    // Log-* aliases
    ['log-fresh', 'log ORIG_HEAD.. --stat --no-merges', 'log', 'show log of new commits after fetch'],
    ['log-list', 'log --graph --topo-order --date=short --abbrev-commit --decorate --all --boundary --pretty=format:\'%Cblue%ad %C(auto)%h%Creset -%C(auto)%d%Creset %s %Cblue[%aN]%Creset %Cblue%G?%Creset\'', 'log', 'log list with preferred information'],
    ['log-list-long', 'log --graph --topo-order --date=iso8601-strict --no-abbrev-commit --decorate --all --boundary --pretty=format:\'%Cblue%ad %C(auto)%h%Creset -%C(auto)%d%Creset %s %Cblue[%aN <%aE>]%Creset %Cblue%G?%Creset\'', 'log', 'log list with long formats'],
    ['log-my', '"!git log --author \\"$(git config user.email)\\"', 'log', 'show log for own commits'],
    ['log-graph', 'log --graph --all --oneline --decorate', 'log', 'show log as a graph'],
    ['log-date-first', '"!git log --date-order --format=%cI | tail -1"', 'log', 'show date of first commit'],
    ['log-date-last', 'log -1 --date-order --format=%cI', 'log', 'show date of last commit'],
    ['log-1-hour', 'log --since=1-hour-ago', 'log', 'log since 1 hour ago'],
    ['log-1-day', 'log --since=1-day-ago', 'log', 'log since 1 day ago'],
    ['log-1-week', 'log --since=1-week-ago', 'log', 'log since 1 week ago'],
    ['log-1-month', 'log --since=1-month-ago', 'log', 'log since 1 month ago'],
    ['log-1-year', 'log --since=1-year-ago', 'log', 'log since 1 year ago'],
    ['log-my-hour', '"!git log --author \\"$(git config user.email)\\" --since=1-hour-ago"', 'log', 'my log since 1 hour ago'],
    ['log-my-day', '"!git log --author \\"$(git config user.email)\\" --since=1-day-ago"', 'log', 'my log since 1 day ago'],
    ['log-my-week', '"!git log --author \\"$(git config user.email)\\" --since=1-week-ago"', 'log', 'my log since 1 week ago'],
    ['log-my-month', '"!git log --author \\"$(git config user.email)\\" --since=1-month-ago"', 'log', 'my log since 1 month ago'],
    ['log-my-year', '"!git log --author \\"$(git config user.email)\\" --since=1-year-ago"', 'log', 'my log since 1 year ago'],
    ['log-refs', 'log --all --graph --decorate --oneline --simplify-by-decoration --no-merges', 'log', 'log refs'],
    ['log-timeline', 'log --format=\'%h %an %ar - %s\'', 'log', 'log timeline'],
    ['log-local', 'log --oneline origin..HEAD', 'log', 'log local commits'],
    ['log-fetched', 'log --oneline HEAD..origin/main', 'log', 'log fetched commits'],

    // Lookup aliases
    ['whois', '"!sh -c \'git log --regexp-ignore-case -1 --pretty=\\"format:%an <%ae>\\" --author=\\"$1\\"\' -"', 'lookup', 'given a string for author, find full name and email'],
    ['whatis', 'show --no-patch --pretty=\'tformat:%h (%s, %ad)\' --date=short', 'lookup', 'given any git object, show it briefly'],
    ['who', 'shortlog --summary --numbered --no-merges', 'lookup', 'show who contributed by number of commits'],
    ['issues', '"!sh -c \\"git log $1 --oneline | grep -o \\\\\\"ISSUE-[0-9]\\\\+\\\\\\" | sort -u\\""', 'lookup', 'list all issues mentioned in commit messages'],
    ['commit-parents', '"!f(){ git cat-file -p \\"${*:-HEAD}\\" | sed -n \'/0/,/^ *$/{/^parent /p}\'; };f"', 'lookup', 'show the commit\'s parents'],
    ['commit-is-merge', '"!f(){ [ -n \\"$(git commit-parents \\"$*\\" | sed \'0,/^parent /d\')\\" ];};f"', 'lookup', 'check if commit is a merge commit'],

    // Workflow aliases
    ['initer', 'init-empty', 'workflow', 'init a repo with empty rebaseable commit'],
    ['cloner', 'clone --recursive', 'workflow', 'clone with recursive submodules'],
    ['clone-lean', 'clone --depth 1 --filter=combine:blob:none+tree:0 --no-checkout', 'workflow', 'clone as lean as possible'],
    ['snapshot', '"!git stash push --include-untracked --message \\"snapshot: $(date)\\" && git stash apply \\"stash@{0}\\" --index"', 'workflow', 'stash snapshot without removing changes'],
    ['panic', '"!tar cvf ../panic.tar -- *"', 'workflow', 'archive everything when panicking'],
    ['archive', '"!f() { top=\\"$(git rev-parse --show-toplevel)\\"; cd \\"$top\\" || exit 1 ; tar cvf \\"$top.tar\\" \\"$top\\" ; }; f"', 'workflow', 'create an archive of everything in repo'],
    ['pushy', 'push --force-with-lease', 'workflow', 'push with force and lease safety check'],
    ['get', '!git fetch --prune && git pull --rebase && git submodule update --init --recursive', 'workflow', 'synchronize all changes - fetch, pull, update submodules'],
    ['put', '!git commit --all && git push', 'workflow', 'commit all and push'],
    ['mainly', '"!git checkout \\"$(git default-branch)\\" && git fetch origin --prune && git reset --hard \\"origin/$(git default-branch)\\""', 'workflow', 'make local repo like the main branch'],
    ['ignore', '"!git status | grep -P \\"^\\\\t\\" | grep -vF .gitignore | sed \\"s/^\\\\t//\\" >> .gitignore"', 'workflow', 'ignore all untracked files'],
    ['push1', '"!git push origin \\"$(git current-branch)\\""', 'workflow', 'push just one branch'],
    ['pull1', '"!git pull origin \\"$(git current-branch)\\""', 'workflow', 'pull just one branch'],
    ['track', '"!f(){ branch=\\"$(git current-branch)\\"; cmd=\\"git branch $branch --set-upstream-to=${1:-origin}/${2:-$branch}\\"; echo \\"$cmd\\"; $cmd; }; f"', 'workflow', 'track with default parameters'],
    ['untrack', '"!f(){ branch=\\"$(git current-branch)\\"; cmd=\\"git branch --unset-upstream ${1:-$branch}\\"; echo \\"$cmd\\"; $cmd; }; f"', 'workflow', 'untrack a branch'],

    // Reset & Undo aliases
    ['reset-commit', 'reset --soft HEAD~1', 'reset', 'reset commit (soft)'],
    ['reset-commit-hard', 'reset --hard HEAD~1', 'reset', 'reset commit (hard)'],
    ['reset-commit-hard-clean', '!git reset --hard HEAD~1 && git clean -fd', 'reset', 'reset commit hard and clean'],
    ['reset-to-pristine', '!git reset --hard && git clean -ffdx', 'reset', 'reset to pristine state'],
    ['reset-to-upstream', '"!git reset --hard \\"$(git upstream-branch)\\""', 'reset', 'reset to upstream branch'],
    ['undo-commit', 'reset --soft HEAD~1', 'undo', 'undo commit (soft)'],
    ['undo-commit-hard', 'reset --hard HEAD~1', 'undo', 'undo commit (hard)'],
    ['undo-commit-hard-clean', '!git reset --hard HEAD~1 && git clean -fd', 'undo', 'undo commit hard and clean'],
    ['undo-to-pristine', '!git reset --hard && git clean -ffdx', 'undo', 'undo to pristine state'],
    ['undo-to-upstream', '"!git reset --hard \\"$(git upstream-branch)\\""', 'undo', 'undo to upstream'],
    ['uncommit', 'reset --soft HEAD~1', 'undo', 'uncommit (soft reset)'],
    ['unadd', 'reset HEAD', 'undo', 'unadd (reset HEAD)'],
    ['discard', 'checkout --', 'undo', 'discard changes in file(s)'],

    // Clean aliases
    ['cleaner', 'clean -dff', 'clean', 'clean with more powerful options'],
    ['cleanest', 'clean -dffx', 'clean', 'clean with most powerful options'],
    ['cleanout', '!git clean -df && git checkout -- .', 'clean', 'clean and checkout'],

    // Expunge
    ['expunge', '"!f() { git filter-branch --force --index-filter \\"git rm --cached --ignore-unmatch $1\\" --prune-empty --tag-name-filter \\"cat\\" -- --all ; }; f"', 'maintenance', 'expunge a file everywhere'],
    ['show-unreachable', '"!git fsck --unreachable | grep commit | cut -d\\" \\" -f3 | xargs git log"', 'maintenance', 'show logs of unreachable commits'],

    // Add-* & Edit-* aliases
    ['add-cached', '"!git add \\"$(git ls-files --cached | sort -u)\\""', 'add', 'add all cached files'],
    ['add-deleted', '"!git add \\"$(git ls-files --deleted | sort -u)\\""', 'add', 'add all deleted files'],
    ['add-others', '"!git add \\"$(git ls-files --others | sort -u)\\""', 'add', 'add all other files'],
    ['add-ignored', '"!git add \\"$(git ls-files --ignored | sort -u)\\""', 'add', 'add all ignored files'],
    ['add-killed', '"!git add \\"$(git ls-files --killed | sort -u)\\""', 'add', 'add all killed files'],
    ['add-modified', '"!git add \\"$(git ls-files --modified | sort -u)\\""', 'add', 'add all modified files'],
    ['add-stage', '"!git add \\"$(git ls-files --stage | cut -f2 | sort -u)\\""', 'add', 'add all stage files'],
    ['add-unmerged', '"!git add \\"$(git ls-files --unmerged | cut -f2 | sort -u)\\""', 'add', 'add all unmerged files'],
    ['edit-cached', '"!$(git var GIT_EDITOR) \\"$(git ls-files --cached | sort -u)\\""', 'edit', 'edit all cached files'],
    ['edit-deleted', '"!$(git var GIT_EDITOR) \\"$(git ls-files --deleted | sort -u)\\""', 'edit', 'edit all deleted files'],
    ['edit-others', '"!$(git var GIT_EDITOR) \\"$(git ls-files --others | sort -u)\\""', 'edit', 'edit all other files'],
    ['edit-ignored', '"!$(git var GIT_EDITOR) \\"$(git ls-files --ignored | sort -u)\\""', 'edit', 'edit all ignored files'],
    ['edit-killed', '"!$(git var GIT_EDITOR) \\"$(git ls-files --killed | sort -u)\\""', 'edit', 'edit all killed files'],
    ['edit-modified', '"!$(git var GIT_EDITOR) \\"$(git ls-files --modified | sort -u)\\""', 'edit', 'edit all modified files'],
    ['edit-stage', '"!$(git var GIT_EDITOR) \\"$(git ls-files --stage | cut -f2 | sort -u)\\""', 'edit', 'edit all stage files'],
    ['edit-unmerged', '"!$(git var GIT_EDITOR) \\"$(git ls-files --unmerged | cut -f2 | sort -u)\\""', 'edit', 'edit all unmerged files'],

    // Ours & Theirs
    ['ours', '"!f() { git checkout --ours \\"$@\\" && git add \\"$@\\"; }; f"', 'merge', 'checkout our version and add it'],
    ['theirs', '"!f() { git checkout --theirs \\"$@\\" && git add \\"$@\\"; }; f"', 'merge', 'checkout their version and add it'],

    // WIP aliases
    ['wip', '"!git add --all; git ls-files --deleted -z | xargs -r -0 git rm; git commit --message=wip"', 'workflow', 'work in progress - add and commit all'],
    ['unwip', '"!git log --max-count=1 | grep -q -c wip && git reset HEAD~1"', 'workflow', 'undo work in progress'],

    // Assume aliases
    ['assume', 'update-index --assume-unchanged', 'assume', 'assume file is unchanged'],
    ['unassume', 'update-index --no-assume-unchanged', 'assume', 'unassume file'],
    ['assume-all', '"!git status --short | awk \'{ print $2 }\' | xargs -r git assume"', 'assume', 'assume all files unchanged'],
    ['unassume-all', '"!git assumed | xargs -r git update-index --no-assume-unchanged"', 'assume', 'unassume all files'],
    ['assumed', '"!git ls-files -v | grep ^h | cut -c 3-"', 'assume', 'list assumed files'],

    // Hew aliases
    ['hew', '"!git hew-local \\"$@\\" && git hew-remote \\"$@\\" #"', 'branch', 'delete all branches merged into a commit'],
    ['hew-dry-run', '"!git hew-local-dry-run \\"$@\\" && git hew-remote-dry-run \\"$@\\" #"', 'branch', 'delete merged branches (dry run)'],
    ['hew-local', '"!f() { git hew-local-dry-run \\"$@\\" | xargs git branch --delete ; }; f \\"$@\\""', 'branch', 'delete local merged branches'],
    ['hew-local-dry-run', '"!f() { commit=${1:-$(git current-branch)}; git branch --merged \\"$commit\\" | grep -v \\"^[[:space:]]*\\\\*[[:space:]]*$commit$\\" ; }; f \\"$@\\""', 'branch', 'delete local merged branches (dry run)'],
    ['hew-remote', '"!f() { git hew-remote-dry-run \\"$@\\" | xargs -I% git push origin :% 2>&1 ; }; f \\"$@\\""', 'branch', 'delete remote merged branches'],
    ['hew-remote-dry-run', '"!f() { commit=${1:-$(git upstream-branch)}; git branch --remotes --merged \\"$commit\\" | grep -v \\"^[[:space:]]*origin/$commit$\\" | sed \'s#[[:space:]]*origin/##\' ; }; f \\"$@\\""', 'branch', 'delete remote merged branches (dry run)'],

    // Publish & Unpublish
    ['publish', '"!f() { git push --set-upstream \\"${1:-origin}\\" \\"$(git current-branch)\\"; }; f"', 'workflow', 'publish current branch to remote'],
    ['unpublish', '"!f() { git push \\"${1:-origin}\\" :\\"$(git current-branch)\\"; }; f"', 'workflow', 'unpublish current branch from remote'],

    // Inbound & Outbound
    ['inbound', '!git remote update --prune; git log ..@{upstream}', 'workflow', 'show incoming changes with upstream'],
    ['outbound', 'log @{upstream}..', 'workflow', 'show outgoing changes with upstream'],

    // Reincarnate
    ['reincarnate', '"!f() { [ $# -gt 0 ] && git checkout \\"$1\\" && git unpublish && git checkout main && git branch --delete --force \\"$1\\" && git checkout -b \\"$1\\" && git publish; }; f"', 'branch', 'delete and recreate branch from main'],

    // Friendly aliases
    ['aliases', '"!git config --get-regexp \'^alias\\\\.\' | cut -c 7- | sed \'s/ / = /\'"', 'alias-management', 'list all aliases'],
    ['branches', 'branch -a', 'branch', 'list all branches'],
    ['tags', 'tag -n1 --list', 'tag', 'list all tags'],
    ['stashes', 'stash list', 'stash', 'list all stashes'],

    // Shell scripting aliases
    ['top', 'rev-parse --show-toplevel', 'shell', 'show top level directory name'],
    ['default-branch', 'config init.defaultBranch', 'shell', 'show default branch name'],
    ['current-branch', 'rev-parse --abbrev-ref HEAD', 'shell', 'show current branch name'],
    ['upstream-branch', '"!git for-each-ref --format=\'%(upstream:short)\' \\"$(git symbolic-ref -q HEAD)\\""', 'shell', 'show upstream branch name'],
    ['upb', 'rev-parse --abbrev-ref "@{upstream}"', 'shell', 'show upstream branch (short)'],
    ['exec', '! exec', 'shell', 'execute shell scripts from top directory'],

    // Maintenance aliases
    ['pruner', '"!git prune --expire=now; git reflog expire --expire-unreachable=now --rewrite --all"', 'maintenance', 'prune everything unreachable'],
    ['repacker', 'repack -a -d -f --depth=300 --window=300 --window-memory=1g', 'maintenance', 'repack the way Linus recommends'],
    ['optimizer', '!git pruner; git repacker; git prune-packed', 'maintenance', 'optimize the repository'],

    // Advanced aliases
    ['search-commits', '"!f() { query=\\"$1\\"; shift; git log -S\\"$query\\" \\"$@\\"; }; f \\"$@\\""', 'advanced', 'search for string in all patches'],
    ['debug', '"!GIT_PAGER=\'\' gdb --args git"', 'advanced', 'debug a git builtin with gdb'],
    ['diff-chunk', '"!f() { git show \\"$1:$3\\" | sed -n \\"/^[^ \\t].*$4(/,/^}/p\\" > .tmp1 ; git show \\"$2:$3\\" | sed -n \\"/^[^ \\t].*$4(/,/^}/p\\" > .tmp2 ; git diff --no-index .tmp1 .tmp2 ; }; f"', 'advanced', 'get the diff of one chunk'],
    ['intercommit', '!sh -c \'git show "$1" > .git/commit1 && git show "$2" > .git/commit2 && interdiff .git/commit[12] | less -FRS\' -', 'advanced', 'interdiff between commits'],
    ['remotes-push', '!git remote | xargs -I% -n1 git push %', 'remote', 'push to all remotes'],
    ['remotes-prune', '!git remote | xargs -n 1 git remote prune', 'remote', 'prune all remotes'],
    ['cherry-pick-merge', '"!sh -c \'git cherry-pick --no-commit --mainline 1 $0 && git log -1 --pretty=%P $0 | cut -b 42- > .git/MERGE_HEAD && git commit --verbose\'"', 'cherry-pick', 'cherry-pick a merge commit'],
    ['rebase-recent', '"!git rebase --interactive \\"$(git remote-ref)\\""', 'rebase', 'rebase recent commits'],
    ['graphviz', '"!f() { echo \'digraph git {\' ; git log --pretty=\'format:  %h -> { %p }\' \\"$@\\" | sed \'s/[0-9a-f][0-9a-f]*/\\"&\\"/g\' ; echo \'}\'; }; f"', 'advanced', 'output graphviz format for visualization'],
    ['serve', '"-c daemon.receivepack=true daemon --base-path=. --export-all --reuseaddr --verbose"', 'advanced', 'serve local directory via git daemon'],

    // Ref aliases
    ['refs-by-date', 'for-each-ref --sort=-committerdate --format=\'%(committerdate:short) %(refname:short) (objectname:short) %(contents:subject)\'', 'ref', 'sort refs by date'],

    // Branch-commit aliases
    ['branch-commit-first', '"!f() { branch=\\"${1:-$(git current-branch)}\\"; count=\\"${2:-1}\\"; git log --reverse --pretty=%H \\"$branch\\" | head -\\"$count\\"; }; f"', 'branch', 'show branch\'s first commit hash(es)'],
    ['branch-commit-last', '"!f() { branch=\\"${1:-$(git current-branch)}\\"; count=\\"${2:-1}\\"; git log --pretty=%H \\"$branch\\" | head -\\"$count\\"; }; f"', 'branch', 'show branch\'s last commit hash(es)'],
    ['branch-commit-prev', '"!f() { branch=\\"${1:-$(git current-branch)}\\"; count=\\"${2:-1}\\"; git log --pretty=%H \\"$branch\\" | grep -A \\"$count\\" \\"$(git rev-parse HEAD)\\" | tail +2; }; f"', 'branch', 'show branch\'s previous commit hash(es)'],
    ['branch-commit-next', '"!f() { branch=\\"${1:-$(git current-branch)}\\"; count=\\"${2:-1}\\"; git log --reverse --pretty=%H \\"$branch\\" | grep -A \\"$count\\" \\"$(git rev-parse HEAD)\\" | tail +2; }; f"', 'branch', 'show branch\'s next commit hash(es)'],

    // Topic branch aliases
    ['topic-base-branch', '"!git config --get init.topicBaseBranchName || git default-branch"', 'topic', 'show topic base branch name'],
    ['topic-start', '"!f(){ new_branch=\\"$1\\"; old_branch=\\"$(git topic-base-branch)\\"; git checkout \\"$old_branch\\"; git pull --ff-only; git checkout -b \\"$new_branch\\" \\"$old_branch\\"; git push --set-upstream origin \\"$new_branch\\"; };f"', 'topic', 'start a topic branch'],
    ['topic-begin', 'topic-start', 'topic', 'begin a topic branch (alias for topic-start)'],
    ['topic-end', '"!f(){ new_branch=\\"$(git current-branch)\\"; old_branch=\\"$(git topic-base-branch)\\"; if [ \\"$new_branch\\" = \\"$old_branch\\" ]; then printf \\"You are on the base topic branch: %s.\\\\n\\" \\"$old_branch\\"; else git push; git checkout \\"$old_branch\\"; git branch --delete \\"$new_branch\\"; git push origin \\":\\"$new_branch\\"\\"; fi; };f"', 'topic', 'stop a topic branch'],
    ['topic-finish', 'topic-end', 'topic', 'finish a topic branch (alias for topic-end)'],
    ['topic-sync', '"!f(){ new_branch=\\"$(git current-branch)\\"; old_branch=\\"$(git topic-base-branch)\\"; if [ \\"$new_branch\\" = \\"$old_branch\\" ]; then printf \\"You are on the base topic branch: %s.\\\\n\\" \\"$old_branch\\"; else git pull; git push; fi; };f"', 'topic', 'sync a topic branch'],
    ['topic-move', '"!f(){ new_branch=\\"$1\\"; old_branch=\\"$(git current-branch)\\"; git branch --move \\"$old_branch\\" \\"$new_branch\\"; git push --set-upstream origin \\":\\"$old_branch\\"\\" \\"$new_branch\\"; };f"', 'topic', 'rename a topic branch'],

    // Integration aliases
    ['cvs-i', 'cvsimport -k -a', 'integration', 'CVS import'],
    ['cvs-e', 'cvsexportcommit -u -p', 'integration', 'CVS export commit'],
    ['gitk-conflict', '!gitk --left-right HEAD...MERGE_HEAD', 'integration', 'show conflicting merge in gitk'],
    ['gitk-history-all', '"!gitk --all \\"$(git fsck | awk \'/dangling commit/ {print $3}\')\\""', 'integration', 'show full history including deleted branches'],
    ['svn-b', 'svn branch', 'integration', 'SVN branch'],
    ['svn-m', 'merge --squash', 'integration', 'SVN merge squash'],
    ['svn-c', 'svn dcommit', 'integration', 'SVN dcommit'],
    ['svn-cp', '!GIT_EDITOR=\'sed -i /^git-svn-id:/d\' git cherry-pick --edit', 'integration', 'SVN cherry-pick'],

    // Churn & Summary (complex shell scripts)
    ['churn', '"!f() { git log --all --find-copies --find-renames --name-only --format=\'format:\' \\"$@\\" | awk \'NF{a[$0]++}END{for(i in a){print a[i], i}}\' | sort -rn;};f"', 'log', 'show log of files that have many changes'],
    ['summary', '"!f() { printf \\"Summary of this branch...\\n\\"; printf \\"%s\\n\\" \\"$(git current-branch)\\"; printf \\"%s first commit\\n\\" \\"$(git log --date-order --format=%cI | tail -1)\\"; printf \\"%s last commit\\n\\" \\"$(git log -1 --date-order --format=%cI)\\"; printf \\"\\nSummary of counts...\\n\\"; printf \\"%d commits\\n\\" \\"$(git rev-list --count HEAD)\\"; }; f"', 'log', 'print a helpful summary of typical metrics'],

    // Track-all
    ['track-all-remote-branches', '"!f() { for x in $(git for-each-ref --format=\\"%(refname:short)\\" --no-merged=origin/HEAD refs/remotes/origin); do git switch --track \\"$x\\"; done; }; f"', 'workflow', 'track all remote branches'],

    // Commit-message
    ['commit-message-key-lines', '"!f(){ echo \\"Commit: $1\\"; git log \\"$1\\" --format=fuller | grep \\"^[[:blank:]]*[[:alnum:]][-[:alnum:]]*:\\" | sed \\"s/^[[:blank:]]*//; s/:[[:blank:]]*/: /\\"; }; f"', 'commit', 'show commit keyword-marked lines'],

    // Chart
    ['chart', '"!f() { git log --format=oneline --format=\\"%aE %at\\" --since=6-weeks-ago \\"$*\\" | awk \'function time_to_slot(t) { return strftime(\\"%Y-%m-%d\\", t, true) } function count_to_char(i) { return (i > 0) ? ((i < 10) ? i : \\"X\\") : \\".\\" } BEGIN { time_min = systime(); time_max = 0; SECONDS_PER_DAY=86400; } { item = $1; time = 0 + $2; if (time > time_max){ time_max = time } else if (time < time_min){ time_min = time }; slot = time_to_slot(time); items[item]++; slots[slot]++; views[item, slot]++; } END{ printf(\\"Chart time range %s to %s.\\\\n\\", time_to_slot(time_min), time_to_slot(time_max)); time_max_add = time_max += SECONDS_PER_DAY; for(item in items){ row = \\"\\"; for(time = time_min; time < time_max_add; time += SECONDS_PER_DAY) { slot = time_to_slot(time); count = views[item, slot]; row = row count_to_char(count); } print row, item; } }\'; }; f"', 'log', 'show a chart of activity per author'],

    // Log-of-* reporting aliases
    ['log-of-format-and-count', '"!f() { format=\\"$1\\"; shift; git log \\"$@\\" --format=oneline --format=\\"$format\\" | awk \'{a[$0]++}END{for(i in a){print i, a[i], int((a[i]/NR)*100) \\"%\\"}}\' | sort; }; f"', 'log', 'show log format and count'],
    ['log-of-count-and-format', '"!f() { format=\\"$1\\"; shift; git log \\"$@\\" --format=oneline --format=\\"$format\\" | awk \'{a[$0]++}END{for(i in a){print a[i], int((a[i]/NR)*100) \\"%\\", i}}\' | sort -nr; }; f"', 'log', 'show log count and format'],
    ['log-of-format-and-count-with-date', '"!f() { format=\\"$1\\"; shift; date_format=\\"$1\\"; shift; git log \\"$@\\" --format=oneline --format=\\"$format\\" --date=format=\\"$date_format\\" | awk \'{a[$0]++}END{for(i in a){print i, a[i], int((a[i]/NR)*100) \\"%\\"}}\' | sort -r; }; f"', 'log', 'show log format and count with date'],
    ['log-of-count-and-format-with-date', '"!f() { format=\\"$1\\"; shift; date_format=\\"$1\\"; shift; git log \\"$@\\" --format=oneline --format=\\"$format\\" --date=format=\\"$date_format\\" | awk \'{a[$0]++}END{for(i in a){print a[i], int((a[i]/NR)*100) \\"%\\", i}}\' | sort -nr; }; f"', 'log', 'show log count and format with date'],
    ['log-of-email-and-count', '"!f() { git log-of-format-and-count \\"%aE\\" \\"$@\\"; }; f"', 'log', 'show log by email and count'],
    ['log-of-count-and-email', '"!f() { git log-of-count-and-format \\"%aE\\" \\"$@\\"; }; f"', 'log', 'show log by count and email'],
    ['log-of-hour-and-count', '"!f() { git log-of-format-and-count-with-date \\"%ad\\" \\"%Y-%m-%dT%H\\" \\"$@\\" ; }; f"', 'log', 'show log by hour and count'],
    ['log-of-count-and-hour', '"!f() { git log-of-count-and-format-with-date \\"%ad\\" \\"%Y-%m-%dT%H\\" \\"$@\\" ; }; f"', 'log', 'show log by count and hour'],
    ['log-of-day-and-count', '"!f() { git log-of-format-and-count-with-date \\"%ad\\" \\"%Y-%m-%d\\" \\"$@\\" ; }; f"', 'log', 'show log by day and count'],
    ['log-of-count-and-day', '"!f() { git log-of-count-and-format-with-date \\"%ad\\" \\"%Y-%m-%d\\" \\"$@\\" ; }; f"', 'log', 'show log by count and day'],
    ['log-of-week-and-count', '"!f() { git log-of-format-and-count-with-date \\"%ad\\" \\"%Y#%V\\" \\"$@\\"; }; f"', 'log', 'show log by week and count'],
    ['log-of-count-and-week', '"!f() { git log-of-count-and-format-with-date \\"%ad\\" \\"%Y#%V\\" \\"$@\\"; }; f"', 'log', 'show log by count and week'],
    ['log-of-month-and-count', '"!f() { git log-of-format-and-count-with-date \\"%ad\\" \\"%Y-%m\\" \\"$@\\" ; }; f"', 'log', 'show log by month and count'],
    ['log-of-count-and-month', '"!f() { git log-of-count-and-format-with-date \\"%ad\\" \\"%Y-%m\\" \\"$@\\" ; }; f"', 'log', 'show log by count and month'],
    ['log-of-year-and-count', '"!f() { git log-of-format-and-count-with-date \\"%ad\\" \\"%Y\\" \\"$@\\" ; }; f"', 'log', 'show log by year and count'],
    ['log-of-count-and-year', '"!f() { git log-of-count-and-format-with-date \\"%ad\\" \\"%Y\\" \\"$@\\" ; }; f"', 'log', 'show log by count and year'],
    ['log-of-hour-of-day-and-count', '"!f() { git log-of-format-and-count-with-date \\"%ad\\" \\"%H\\" \\"$@\\"; }; f"', 'log', 'show log by hour of day and count'],
    ['log-of-count-and-hour-of-day', '"!f() { git log-of-count-and-format-with-date \\"%ad\\" \\"%H\\" \\"$@\\"; }; f"', 'log', 'show log by count and hour of day'],
    ['log-of-day-of-week-and-count', '"!f() { git log-of-format-and-count-with-date \\"%ad\\" \\"%u\\" \\"$@\\"; }; f"', 'log', 'show log by day of week and count'],
    ['log-of-count-and-day-of-week', '"!f() { git log-of-count-and-format-with-date \\"%ad\\" \\"%u\\" \\"$@\\"; }; f"', 'log', 'show log by count and day of week'],
    ['log-of-week-of-year-and-count', '"!f() { git log-of-format-and-count-with-date \\"%ad\\" \\"%V\\" \\"$@\\"; }; f"', 'log', 'show log by week of year and count'],
    ['log-of-count-and-week-of-year', '"!f() { git log-of-count-and-format-with-date \\"%ad\\" \\"%V\\" \\"$@\\"; }; f"', 'log', 'show log by count and week of year'],

    // Remote-ref
    ['remote-ref', '"!local_ref=\\"$(git symbolic-ref HEAD)\\"; local_name=\\"${local_ref##refs/heads/}\\"; remote=\\"$(git config branch.\\"$local_name\\".remote || echo origin)\\"; remote_ref=\\"$(git config branch.\\"$local_name\\".merge)\\"; remote_name=\\"${remote_ref##refs/heads/}\\"; echo \\"remotes/$remote/$remote_name\\" #"', 'remote', 'show remote ref for current branch'],

    // Rev-list extended
    ['rev-list-all-objects-by-size-and-name', '"!git rev-list --all --objects | git cat-file --batch-check=\'%(objecttype) %(objectname) %(objectsize) %(rest)\' | awk \'/^blob/ {print substr($0,6)}\' | sort --numeric-sort --key=2"', 'maintenance', 'list all blobs by size and file name'],
];

// ── Build typed array from raw data ─────────────────────────────

export const LIBRARY: I_LibraryAlias[] = RAW.map(([name, command, category, description]) => ({
    name,
    command,
    category,
    description,
}));
