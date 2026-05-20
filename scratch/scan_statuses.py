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
    "draft",
    "ready_to_implement",
    "implementing",
    "ready_to_review",
    "reviewing",
    "ready_to_test",
    "testing",
    "done",
    "on_hold",
    "closed"
}

def scan_project(proj, out_file):
    base_dir = f"../{proj}"
    if not os.path.exists(base_dir):
        out_file.write(f"Directory {base_dir} does not exist.\n")
        return
    
    out_file.write(f"\n=========================================\n")
    out_file.write(f"PROJECT: {proj}\n")
    out_file.write(f"=========================================\n")
    
    md_files = []
    for root, dirs, files in os.walk(base_dir):
        for f in files:
            if f.endswith(".md"):
                full_path = os.path.join(root, f)
                if f.startswith("FR-") and not f.endswith(".audit.md"):
                    md_files.append(full_path)
                elif f == "BACKLOG.md" or f == "BACKLOG_INDEX.md":
                    md_files.append(full_path)
                
    for path in sorted(md_files):
        try:
            with open(path, "r", encoding="utf-8") as file:
                content = file.read()
        except Exception as e:
            out_file.write(f"  Error reading {path}: {e}\n")
            continue
            
        if os.path.basename(path).startswith("FR-"):
            match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
            if match:
                fm_text = match.group(1)
                try:
                    fm = yaml.safe_load(fm_text)
                    fr_id = fm.get("id")
                    status = fm.get("status")
                    if status not in CANONICAL_STATUSES:
                        out_file.write(f"  FR: {path} | ID: {fr_id} | Non-canonical Status: {status}\n")
                    else:
                        out_file.write(f"  FR: {path} | ID: {fr_id} | Canonical Status: {status}\n")
                except Exception as e:
                    status_match = re.search(r"^status:\s*(.*?)$", fm_text, re.MULTILINE)
                    if status_match:
                        status = status_match.group(1).strip()
                        if status not in CANONICAL_STATUSES:
                            out_file.write(f"  FR (yaml err fallback): {path} | Status: {status}\n")
                        else:
                            out_file.write(f"  FR (yaml err fallback): {path} | Status: {status} (canonical)\n")
                    else:
                        out_file.write(f"  YAML Error and no status line in {path}: {e}\n")
            else:
                status_match = re.search(r"^status:\s*(.*?)$", content[:1000], re.MULTILINE)
                if status_match:
                    status = status_match.group(1).strip()
                    if status not in CANONICAL_STATUSES:
                        out_file.write(f"  FR (no fm boundary fallback): {path} | Status: {status}\n")
                    else:
                        out_file.write(f"  FR (no fm boundary fallback): {path} | Status: {status} (canonical)\n")
                else:
                    out_file.write(f"  No frontmatter or status field in: {path}\n")
        else:
            out_file.write(f"  Backlog file: {path}\n")

if __name__ == "__main__":
    report_path = "/Users/stephencheng/.gemini/antigravity/brain/b570e0d8-42b1-442c-9c4f-124f2bca4f91/scratch/statuses_report.txt"
    with open(report_path, "w", encoding="utf-8") as out_file:
        for p in PROJECTS:
            scan_project(p, out_file)
    print(f"Report written to {report_path}")
