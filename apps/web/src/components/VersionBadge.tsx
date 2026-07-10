import { useEffect, useState } from "react";
import { t } from "../lib/i18n";
import { useUpdateCheck } from "../lib/useUpdateCheck";

// Small topbar badge: shows the running CyberOS version, and turns into an "update available -> reload"
// affordance the moment a newer build is live. The version string comes from /version.json (written by
// scripts/stamp-sw.mjs on every build); the pending flag reuses useUpdateCheck (the same poll that drives
// the reload banner), so there is one source of truth for "is a newer build deployed".
export function VersionBadge() {
  const updateReady = useUpdateCheck();
  const [version, setVersion] = useState<string>("");

  useEffect(() => {
    let alive = true;
    fetch(`${import.meta.env.BASE_URL}version.json?t=${Date.now()}`, { cache: "no-store" })
      .then((r) => (r.ok ? r.json() : null))
      .then((body: unknown) => {
        const v = body && typeof body === "object" ? (body as { version?: unknown }).version : null;
        if (alive && typeof v === "string" && v.length > 0) setVersion(v);
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, []);

  if (updateReady) {
    return (
      <button
        type="button"
        className="version-badge version-badge-update"
        title={t("update.available")}
        onClick={() => location.reload()}
      >
        <span className="version-dot" aria-hidden="true" />
        {t("update.reload")}
      </button>
    );
  }

  if (!version) return null;
  return (
    <span className="version-badge" title={t("version.current")}>
      v{version}
    </span>
  );
}
