import { command } from "./bridge";
import type { LightNovelUser, LoginInput, RegisterInput } from "../domain/auth";

export type { LightNovelUser, LoginInput, RegisterInput } from "../domain/auth";

export const login = (input: LoginInput) => {
  return command<LightNovelUser>("login", { ...input });
}

export const register = (input: RegisterInput) => {
  return command<LightNovelUser>("register", { ...input });
}

export const sendRegisterEmail = (email: string) => {
  return command<void>("send_register_email", { email });
}

export const restoreUser = () => {
  return command<LightNovelUser | null>("restore_user");
}

export const setAvatar = (url: string) => {
  return command<LightNovelUser>("set_avatar", { url });
}

export const signIn = () => {
  return command<LightNovelUser>("sign_in");
}

export const logout = () => {
  return command<void>("logout");
}
