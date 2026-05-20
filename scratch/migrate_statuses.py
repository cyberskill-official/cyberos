import os
import re

PROJECTS = [
    "design-system-audit-framework",
    "landing-page",
    "sale-noti",
    "tamagochi",
    "styx",
    "cyberos"
]

CANONICAL_STATUSES = {
    "draft", "ready_to_implement", "implementing", "ready_to_review",
    "reviewing", "ready_to_test", "testing", "done", "on_hold", "closed"
}

def map_status(s_str):
    if not s_str:
        return None
    s = s_str.strip().lower()
    
    # Strip any enclosing quotes
    if (s.startswith("'") and s.endswith("'")) or (s.startswith('"') and s.endswith('"')):
        s = s[1:-1].strip()
        
    if s in CANONICAL_STATUSES:
        return s
    
    # Mapping logic per STATUS-REFERENCE.md
    if any(x in s for x in ["shipped", "done", "signed"]):
        return "done"
    if any(x in s for x in ["blocked", "accepted", "audited", "in_review", "planned", "ready"]):
        return "ready_to_implement"
    if any(x in s for x in ["building", "in_progress", "implementing"]):
        return "implementing"
    if any(x in s for x in ["deferred", "on_hold"]):
        return "on_hold"
    if any(x in s for x in ["rejected", "superseded", "closed"]):
        return "closed"
        
    return "ready_to_implement"

def process_file_frontmatter(path, dry_run=True):
    try:
        with open(path, "r", encoding="utf-8") as file:
            content = file.read()
    except Exception as e:
        print(f"Error reading {path}: {e}")
        return False

    # Check for frontmatter
    match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
    if not match:
        return False

    fm_text = match.group(1)
    fm_lines = fm_text.splitlines()
    changed = False
    new_fm_lines = []
    
    status_mapped = None
    
    for line in fm_lines:
        status_match = re.match(r"^status:\s*(.*)$", line)
        lifecycle_match = re.match(r"^lifecycle:\s*(.*)$", line)
        
        if status_match:
            val = status_match.group(1).strip()
            clean_val = val
            if (clean_val.startswith("'") and clean_val.endswith("'")) or (clean_val.startswith('"') and clean_val.endswith('"')):
                clean_val = clean_val[1:-1].strip()
            
            if clean_val not in CANONICAL_STATUSES:
                status_mapped = map_status(clean_val)
                line = f"status: {status_mapped}"
                changed = True
                print(f"  Frontmatter status in {os.path.relpath(path)}: '{val}' -> '{status_mapped}'")
        elif lifecycle_match:
            val = lifecycle_match.group(1).strip()
            clean_val = val
            if (clean_val.startswith("'") and clean_val.endswith("'")) or (clean_val.startswith('"') and clean_val.endswith('"')):
                clean_val = clean_val[1:-1].strip()
                
            if clean_val not in CANONICAL_STATUSES:
                mapped = map_status(clean_val)
                line = f"lifecycle: {mapped}"
                changed = True
                print(f"  Frontmatter lifecycle in {os.path.relpath(path)}: '{val}' -> '{mapped}'")
                
        new_fm_lines.append(line)

    if status_mapped == "done":
        original_status_val = None
        for line in fm_lines:
            status_match = re.match(r"^status:\s*(.*)$", line)
            if status_match:
                original_status_val = status_match.group(1).strip().lower()
                break
        if original_status_val and "mocked" in original_status_val:
            has_impl_kind = any(re.match(r"^implementation_kind:", l) for l in new_fm_lines)
            if not has_impl_kind:
                new_fm_lines.append("implementation_kind: mocked")
                changed = True
                print(f"  Added implementation_kind: mocked to {os.path.relpath(path)}")

    if changed:
        new_fm_text = "\n".join(new_fm_lines)
        new_content = content.replace(fm_text, new_fm_text, 1)
        if not dry_run:
            with open(path, "w", encoding="utf-8") as file:
                file.write(new_content)
        return True

    return False

def process_backlog_table(path, dry_run=True):
    try:
        with open(path, "r", encoding="utf-8") as file:
            content = file.read()
    except Exception as e:
        print(f"Error reading backlog {path}: {e}")
        return False

    lines = content.splitlines()
    changed = False
    status_col_idx = -1
    new_lines = []
    
    for idx, line in enumerate(lines, 1):
        if "|" in line:
            parts = line.split("|")
            stripped_parts = [p.strip() for p in parts]
            
            # Check if this is a header line
            if "status" in [p.lower() for p in stripped_parts]:
                lower_parts = [p.lower() for p in stripped_parts]
                status_col_idx = lower_parts.index("status")
                new_lines.append(line)
                continue
                
            if status_col_idx != -1 and len(parts) > status_col_idx:
                fr_id_part = stripped_parts[1].replace("**", "").replace("*", "").strip()
                if fr_id_part.startswith("FR-") or fr_id_part.startswith("STX-"):
                    original_status_cell = parts[status_col_idx]
                    clean_status = stripped_parts[status_col_idx].replace("**", "").replace("*", "").strip()
                    
                    mapped = map_status(clean_status)
                    
                    if mapped != clean_status:
                        # Replace the status column cell, preserving leading/trailing space for neatness
                        parts[status_col_idx] = f" {mapped} "
                        new_line = "|".join(parts)
                        
                        print(f"  Backlog Line {idx} in {os.path.relpath(path)}: '{original_status_cell.strip()}' -> '{mapped}'")
                        new_lines.append(new_line)
                        changed = True
                        continue
        
        new_lines.append(line)

    if changed:
        new_content = "\n".join(new_lines) + ("\n" if content.endswith("\n") else "")
        if not dry_run:
            with open(path, "w", encoding="utf-8") as file:
                file.write(new_content)
        return True
        
    return False

def migrate_all(dry_run=True):
    print(f"Starting status migration. Dry run = {dry_run}")
    for proj in PROJECTS:
        base_dir = f"../{proj}"
        if not os.path.exists(base_dir):
            continue
        print(f"\nProcessing project: {proj}")
        for root, dirs, files in os.walk(base_dir):
            if any(x in root for x in [".git", "node_modules", "dist", "build", "target", "apps", "services", "src", "packages", "modules"]):
                continue
            for f in files:
                if f.endswith(".md"):
                    path = os.path.join(root, f)
                    rel_path = os.path.relpath(path, base_dir)
                    is_task_or_fr = (
                        f.startswith("FR-") or 
                        f.startswith("STX-") or 
                        "feature-requests" in rel_path or 
                        "tasks" in rel_path
                    )
                    
                    if is_task_or_fr:
                        process_file_frontmatter(path, dry_run=dry_run)
                        
                    if f in ["BACKLOG.md", "BACKLOG_INDEX.md"]:
                        process_backlog_table(path, dry_run=dry_run)

if __name__ == "__main__":
    import sys
    dry = True
    if len(sys.argv) > 1 and sys.argv[1] == "run":
        dry = False
    migrate_all(dry_run=dry)
