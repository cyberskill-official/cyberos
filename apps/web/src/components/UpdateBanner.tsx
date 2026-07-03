import { t } from "../lib/i18n";
import { useUpdateCheck } from "../lib/useUpdateCheck";

// Non-blocking prompt shown when a newer build has been deployed. Bottom-centered like the undo toast, but it
// persists (no countdown) since applying it is the user's choice. Reload pulls the new build in.
export function UpdateBanner() {
  const ready = useUpdateCheck();
  if (!ready) return null;
  return (
    <div className="update-banner" role="status" aria-live="polite">
      <span className="update-msg">{t("update.available")}</span>
      <button type="button" className="update-reload" onClick={() => location.reload()}>
        {t("update.reload")}
      </button>
    </div>
  );
}
