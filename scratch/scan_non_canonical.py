import os
import re
import yaml

PROJECTS = [
    "design-system-audit-framework",
    "landing-page",
    "sale-noti",
    "tamagochi",
    "styx"
]

CANONICAL_STATUSES = {
    "draft", "ready_to_implement", "implementing", "ready_to_review",
    "reviewing", "ready_to_test", "testing", "done", "on_hold", "closed"
}

def scan():
    output_file = "/Users/stephencheng/.gemini/antigravity/brain/b570e0d8-42b1-442c-9c4f-124f2bca4f91/scratch/non_canonical_report.txt"
    with open(output_file, "w", encoding="utf-8") as out:
        for proj in PROJECTS:
            base_dir = f"../{proj}"
            if not os.path.exists(base_dir):
                out.write(f"Directory {base_dir} does not exist.\n")
                continue
            out.write(f"\n=========================================\nPROJECT: {proj}\n=========================================\n")
            for root, dirs, files in os.walk(base_dir):
                if any(x in root for x in [".git", "node_modules", "dist", "build", "target", "apps", "services", "src", "packages", "modules"]):
                    continue
                for f in files:
                    if f.endswith(".md"):
                        path = os.path.join(root, f)
                        try:
                            with open(path, "r", encoding="utf-8") as file:
                                content = file.read()
                        except Exception as e:
                            continue
                        
                        # Extract frontmatter
                        match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
                        if match:
                            fm_text = match.group(1)
                            try:
                                fm = yaml.safe_load(fm_text)
                                if isinstance(fm, dict):
                                    status = fm.get("status")
                                    lifecycle = fm.get("lifecycle")
                                    if status and status not in CANONICAL_STATUSES:
                                        out.write(f"FR File with non-canonical status: {path} | status: {status}\n")
                                    if lifecycle and lifecycle not in CANONICAL_STATUSES:
                                        out.write(f"FR File with non-canonical lifecycle: {path} | lifecycle: {lifecycle}\n")
                            except Exception:
                                pass
                        
                        # Scan BACKLOG.md/BACKLOG_INDEX.md
                        if f in ["BACKLOG.md", "BACKLOG_INDEX.md"]:
                            out.write(f"Backlog File: {path}\n")
                            lines = content.splitlines()
                            for idx, line in enumerate(lines, 1):
                                if "|" in line:
                                    if re.search(r"\|\s*\*\*?FR-", line) or re.search(r"\|\s*\*\*?STX-", line):
                                        parts = [p.strip() for p in line.split("|")]
                                        found_non_canonical = False
                                        non_canonical_found_word = ""
                                        for part in parts:
                                            part_clean = part.replace("**", "").replace("*", "").strip()
                                            for word in ["shipped", "accepted", "building", "blocked", "audited", "deferred", "in_progress", "superseded"]:
                                                if word in part_clean.lower() and not any(c in part_clean.lower() for c in CANONICAL_STATUSES if c != "draft"):
                                                    found_non_canonical = True
                                                    non_canonical_found_word = part_clean
                                                    break
                                                if "shipped" in part_clean.lower() or "blocked" in part_clean.lower() or "accepted" in part_clean.lower():
                                                    found_non_canonical = True
                                                    non_canonical_found_word = part_clean
                                                    break
                                        if found_non_canonical:
                                            out.write(f"  Line {idx}: {line[:120]}... | Found status text: '{non_canonical_found_word}'\n")

if __name__ == "__main__":
    scan()
