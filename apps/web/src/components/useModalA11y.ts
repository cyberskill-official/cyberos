import { useEffect, useRef } from "react";

// Every focusable a natural Tab order would visit, used to trap Tab inside a dialog and to pick where focus
// lands on open.
const FOCUSABLE =
  'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';

// Adds the shared accessible-modal behaviors to a dialog box: focus moves inside on open, Escape closes, Tab
// and Shift+Tab are trapped within, and focus returns to whatever was focused before (the trigger) on close.
// Attach the returned ref to the dialog element (the `.picker` box, given tabIndex={-1} so it can hold focus).
// The backdrop-click-to-close stays in each modal's own markup. Behaviors are additive - no markup restructure.
export function useModalA11y<T extends HTMLElement = HTMLDivElement>(onClose: () => void) {
  const boxRef = useRef<T | null>(null);
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  useEffect(() => {
    const box = boxRef.current;
    const restore = document.activeElement as HTMLElement | null;
    if (!box) {
      return () => restore?.focus?.();
    }
    // Move focus into the dialog (first focusable, else the box itself).
    const initial = box.querySelectorAll<HTMLElement>(FOCUSABLE);
    (initial[0] || box).focus();

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        onCloseRef.current();
        return;
      }
      if (e.key !== "Tab") return;
      const els = Array.from(box.querySelectorAll<HTMLElement>(FOCUSABLE));
      if (els.length === 0) return;
      const first = els[0];
      const last = els[els.length - 1];
      const active = document.activeElement;
      if (e.shiftKey && active === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && active === last) {
        e.preventDefault();
        first.focus();
      }
    };
    box.addEventListener("keydown", onKey);
    return () => {
      box.removeEventListener("keydown", onKey);
      // Return focus to the trigger so keyboard users are not dropped at the top of the page.
      restore?.focus?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return boxRef;
}
