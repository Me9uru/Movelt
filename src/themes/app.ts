import { Themes } from "@varlet/ui";

/** Shared MD3 component geometry used by every application colour scheme. */
export const materialComponentTheme = {
  "--button-border-radius": "999px",
  "--button-normal-height": "40px",
  "--button-large-height": "48px",
  "--button-small-height": "32px",
  "--card-border-radius": "16px",
  "--chip-border-radius": "8px",
  "--chip-round-radius": "8px",
  "--field-decorator-line-border-radius": "8px",
  "--field-decorator-filled-border-radius": "8px",
  "--snackbar-border-radius": "8px",
  "--segmented-buttons-border-radius": "999px",
  "--segmented-button-normal-height": "40px",
  "--tabs-item-horizontal-height": "48px",
  "--tabs-indicator-border-radius": "999px 999px 0 0",
  "--tab-font-size": "14px",
  "--tab-font-weight": "500",
  "--tab-active-font-size": "14px",
  "--tab-active-font-weight": "600",
} as const;

/** Movel's blue MD3 light colour scheme. */
export const lightTheme = {
  ...Themes.md3Light,
  ...materialComponentTheme,
  "--hsl-body": "207, 56%, 96%",
  "--hsl-text": "208, 43%, 14%",
  "--hsl-primary": "208, 69%, 50%",
  "--hsl-info": "207, 36%, 42%",
  "--hsl-primary-container": "207, 100%, 94%",
  "--hsl-info-container": "207, 58%, 91%",
  "--hsl-on-primary-container": "207, 82%, 17%",
  "--hsl-on-info-container": "207, 54%, 18%",
  "--hsl-surface-container": "207, 48%, 94%",
  "--hsl-surface-container-low": "207, 60%, 97%",
  "--hsl-surface-container-high": "207, 38%, 91%",
  "--hsl-surface-container-highest": "207, 30%, 88%",
  "--hsl-inverse-surface": "207, 20%, 20%",
  "--hsl-outline": "207, 25%, 72%",
  "--hsl-on-surface-variant": "207, 21%, 32%",
  "--popup-overlay-background-color": "rgb(25 49 68 / 42%)",
} as const;

/** Movel's blue MD3 dark colour scheme. */
export const nightTheme = {
  ...Themes.md3Dark,
  ...materialComponentTheme,
  "--hsl-body": "207, 19%, 12%",
  "--hsl-text": "207, 24%, 90%",
  "--hsl-primary": "206, 76%, 69%",
  "--hsl-info": "206, 36%, 72%",
  "--hsl-primary-container": "206, 46%, 28%",
  "--hsl-info-container": "206, 28%, 27%",
  "--hsl-on-primary": "206, 100%, 16%",
  "--hsl-on-primary-container": "206, 100%, 91%",
  "--hsl-on-info-container": "206, 54%, 89%",
  "--hsl-surface-container": "207, 18%, 17%",
  "--hsl-surface-container-low": "207, 18%, 15%",
  "--hsl-surface-container-high": "207, 17%, 20%",
  "--hsl-surface-container-highest": "207, 16%, 24%",
  "--hsl-inverse-surface": "207, 22%, 88%",
  "--hsl-outline": "207, 16%, 38%",
  "--hsl-on-surface-variant": "207, 16%, 72%",
  "--popup-overlay-background-color": "rgb(0 0 0 / 56%)",
} as const;
