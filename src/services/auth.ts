import { command } from "./bridge";

export interface LightNovelUser {
  Id: number;
  UserName: string;
  Avatar?: string;
  Email?: string;
}

export function login(email: string, password: string) {
  return command<LightNovelUser>("login", { email, password });
}

export function register(userName: string, email: string, password: string, code: string, inviteCode = "") {
  return command<LightNovelUser>("register", { userName, email, password, code, inviteCode });
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
