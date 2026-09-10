/** Shared keyboard focus handling for the application's custom popup content. */
export const useDialogFocus = (root: () => HTMLElement | null) => {
  let opener: HTMLElement | null = null;
  const rememberFocus = (): void => {
    opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  };
  const restoreFocus = (): void => {
    if (opener?.isConnected) opener.focus({ preventScroll: true });
  };
  const trapFocus = (event: KeyboardEvent): void => {
    if (event.key !== "Tab") return;
    const controls = Array.from(root()?.querySelectorAll<HTMLElement>(
      "button:not(:disabled), input:not(:disabled), [tabindex='0']",
    ) ?? []).filter((element) => element.getClientRects().length > 0);
    const first = controls[0];
    const last = controls[controls.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last?.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first?.focus();
    }
  };
  return { rememberFocus, restoreFocus, trapFocus };
};
