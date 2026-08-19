import FingerprintJS from "@fingerprintjs/fingerprintjs";
import localforage from "localforage";

import { rebootHub, requestWithFetch, requestWithHub } from "./lightnovel";

const apiServer = "https://api.lightnovel.life";
const refreshStore = localforage.createInstance({ name: "movel", storeName: "authentication" });
const refreshKey = "refresh-token";
let sessionToken = "";
let sessionExpiresAt = 0;

export interface LightNovelUser {
  Id: number;
  UserName: string;
  Avatar?: string;
}

async function visitorId(): Promise<string> {
  const fingerprint = await FingerprintJS.load();
  return (await fingerprint.get()).visitorId;
}

async function sha256(value: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

export function apiBaseUrl(): string {
  return apiServer;
}

export async function login(email: string, password: string): Promise<LightNovelUser> {
  const response = await requestWithFetch<{ Token: string; RefreshToken: string }>("/api/user/login", {
    email,
    password: await sha256(password),
  }, await visitorId());
  sessionToken = response.Token;
  sessionExpiresAt = Date.now() + 3_000;
  await refreshStore.setItem(refreshKey, response.RefreshToken);
  await rebootHub();
  return requestWithHub<LightNovelUser>("GetMyInfo");
}

export async function getSessionToken(): Promise<string> {
  if (sessionToken && Date.now() < sessionExpiresAt) return sessionToken;
  const refreshToken = await refreshStore.getItem<string>(refreshKey);
  if (!refreshToken) return "";
  try {
    sessionToken = await requestWithFetch<string>("/api/user/refresh_token", { token: refreshToken }, await visitorId());
    sessionExpiresAt = Date.now() + 3_000;
    return sessionToken;
  } catch {
    await refreshStore.removeItem(refreshKey);
    sessionToken = "";
    sessionExpiresAt = 0;
    return "";
  }
}

export async function restoreUser(): Promise<LightNovelUser | null> {
  if (!(await getSessionToken())) return null;
  try {
    return await requestWithHub<LightNovelUser>("GetMyInfo");
  } catch {
    return null;
  }
}

export async function logout(): Promise<void> {
  sessionToken = "";
  sessionExpiresAt = 0;
  await refreshStore.removeItem(refreshKey);
  await rebootHub();
}
