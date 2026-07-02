import { useEffect, useRef } from "react";
import { t } from "../lib/i18n";

// An accessible in-app replacement for window.confirm: a focus-trapped alertdialog where Escape and the
// backdrop cancel, Tab cycles between the two buttons, and Cancel is auto-focused (the safe default for a
// destructive action, so a reflexive Enter/Escape never triggers it). The primary button carries the verb.
export function ConfirmDialog({
  body,
  confirmLabel,
  danger = true,
  busy = false,
  onConfirm,
  onCancel,
}: {
  body: string;
  confirmLabel: string;
  danger?: boolean;
  busy?: boolean;
  onConfirm(): void;
  onCancel(): void;
}) {
  const cancelRef = useRef<HTMLButtonElement | null>(null);
  const boxRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    cancelRef.current?.focus();
  }, []);

  return (
    <div className="picker-bg confirm-bg" onClick={(e) => e.target === e.currentTarget && !busy && onCancel()}>
      <div
        className="confirm-box"
        role="alertdialog"
        aria-modal="true"
        aria-label={t("confirm.title")}
        ref={boxRef}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            e.preventDefault();
            if (!busy) onCancel();
          } else if (e.key === "Tab") {
            // Two-button focus trap: keep Tab / Shift+Tab inside the dialog.
            const els = boxRef.current?.querySelectorAll<HTMLButtonElement>("button:not([disabled])");
            if (!els || els.length === 0) return;
            const first = els[0];
            const last = els[els.length - 1];
            if (e.shiftKey && document.activeElement === first) {
              e.preventDefault();
              last.focus();
            } else if (!e.shiftKey && document.activeElement === last) {
              e.preventDefault();
              first.focus();
            }
          }
        }}
      >
        <div className="confirm-body">{body}</div>
        <div className="confirm-actions">
          <button className="btn-ghost" ref={cancelRef} onClick={onCancel} disabled={busy} type="button">
            {t("common.cancel")}
          </button>
          <button
            className={"btn-pill" + (danger ? " danger" : "")}
            onClick={onConfirm}
            disabled={busy}
            type="button"
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
