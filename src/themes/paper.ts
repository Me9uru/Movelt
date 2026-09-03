import { Themes } from "@varlet/ui";
import { materialComponentTheme } from "./app";

/**
 * A warm, paper-like MD3 palette. It extends Varlet's MD3 light theme so every
 * component-specific token remains present while the shared semantic colours
 * are tailored for the reader's paper theme.
 */
export const paperTheme = {
  ...Themes.md3Light,
  ...materialComponentTheme,
  "--color-scheme": "light",
  "--hsl-body": "42, 36%, 94%",
  "--hsl-text": "36, 10%, 20%",
  "--hsl-primary": "31, 22%, 43%",
  "--hsl-info": "28, 18%, 42%",
  "--hsl-success": "88, 24%, 31%",
  "--hsl-warning": "24, 47%, 39%",
  "--hsl-danger": "6, 48%, 42%",
  "--hsl-primary-container": "31, 38%, 88%",
  "--hsl-info-container": "29, 31%, 88%",
  "--hsl-success-container": "86, 35%, 84%",
  "--hsl-warning-container": "25, 62%, 88%",
  "--hsl-danger-container": "8, 55%, 89%",
  "--hsl-on-primary-container": "30, 30%, 17%",
  "--hsl-on-info-container": "28, 24%, 18%",
  "--hsl-on-success-container": "87, 28%, 14%",
  "--hsl-on-warning-container": "24, 42%, 17%",
  "--hsl-on-danger-container": "6, 43%, 17%",
  "--hsl-surface-container": "40, 31%, 92%",
  "--hsl-surface-container-low": "42, 42%, 97%",
  "--hsl-surface-container-high": "38, 24%, 89%",
  "--hsl-surface-container-highest": "35, 18%, 85%",
  "--hsl-inverse-surface": "35, 8%, 24%",
  "--hsl-outline": "34, 14%, 72%",
  "--hsl-on-surface-variant": "34, 10%, 34%",
  "--popup-overlay-background-color": "rgb(52 45 36 / 52%)",
} as const;
