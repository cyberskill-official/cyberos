#!/usr/bin/env bash
# Regression set for harden-plan.mjs. The five pilot reports are the fixtures;
# each assertion below was verified by hand once and must hold thereafter.
set -u
# Resolve everything against this script's own location so the set runs from
# anywhere, including from inside the packaged skill tree. A regression set that
# only passes from one working directory is not shipped, it is coincidental.
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLAN="$HERE/harden-plan.mjs"
FIX="$HERE/../../inspection-report-audit/acceptance"
if [ ! -d "$FIX" ]; then echo "fixtures not found at $FIX"; exit 2; fi
fail=0
check() { # name expected actual
  if [ "$2" = "$3" ]; then printf "  ok   %-46s %s\n" "$1" "$3"
  else printf "  FAIL %-46s expected %s got %s\n" "$1" "$2" "$3"; fail=1; fi
}
j() { node "$PLAN" "$FIX/$1.golden.md" --json; }
rep() { echo "$FIX/$1.golden.md"; }

# Entry point resolves and is the report's own NEXT-ACTION.
for r in my-cv issue-hunter dom-defender gam kristen-calendar; do
  want=$(grep -oE '^NEXT-ACTION: (INS-F-[0-9]{4})' "$(rep "$r")" | awk '{print $2}')
  got=$(j "$r" | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.parse(s).ordered[0].id))')
  check "$r entry point" "$want" "$got"
done

# Actor classification: the six findings an agent cannot fully close.
check "issue-hunter non-agent count" 3 "$(j issue-hunter | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.parse(s).ordered.filter(o=>o.actor!=="agent").length))')"
check "kristen-calendar non-agent count" 2 "$(j kristen-calendar | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.parse(s).ordered.filter(o=>o.actor!=="agent").length))')"
check "gam non-agent count" 1 "$(j gam | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.parse(s).ordered.filter(o=>o.actor!=="agent").length))')"
check "dom-defender all agent" 0 "$(j dom-defender | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.parse(s).ordered.filter(o=>o.actor!=="agent").length))')"
check "my-cv all agent" 0 "$(j my-cv | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.parse(s).ordered.filter(o=>o.actor!=="agent").length))')"

# The credential rotation must never be classified agent.
check "kristen-calendar entry is split" split "$(j kristen-calendar | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.parse(s).ordered[0].actor))')"

# Deferred-class findings are held, not ordered.
check "my-cv held count" 2 "$(j my-cv | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.parse(s).held.length))')"

# operator_prerequisites must force non-agent (GUIDELINE §7 / TASK-IMP-147).
# shopass.r2 INS-F-0002 needs a DBA + secret-manager update — never pure agent.
if [ -f "$FIX/shopass.r2.golden.md" ]; then
  check "shopass.r2 INS-F-0002 not agent" 1 "$(j shopass.r2 | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{const o=JSON.parse(s).ordered.find(x=>x.id==="INS-F-0002"); console.log(o&&o.actor!=="agent"?1:0)})')"
else
  printf "  FAIL %-46s %s\n" "shopass.r2 fixture present" "missing $FIX/shopass.r2.golden.md"
  fail=1
fi

# A report with no NEXT-ACTION is refused.
sed '/^NEXT-ACTION:/d' "$(rep gam)" > /tmp/no-na.md
node "$PLAN" /tmp/no-na.md >/dev/null 2>&1
check "refuses a report with no next action" 2 "$?"

[ $fail -eq 0 ] && echo "PASS" || echo "FAIL"
exit $fail
