import { command } from "./bridge";
import type { LightNovelUser, LoginInput, RegisterInput } from "../domain/auth";

export type { LightNovelUser, LoginInput, RegisterInput } from "../domain/auth";

export function login(input: LoginInput) {
  return command<LightNovelUser>("login", { ...input });
}

export function register(input: RegisterInput) {
  return command<LightNovelUser>("register", { ...input });
}

export function sendRegisterEmail(email: string) {
  return command<void>("send_register_email", { email });
}

export function restoreUser() {
  return command<LightNovelUser | null>("restore_user");
}

export function logout() {
  return command<void>("logout");
}
