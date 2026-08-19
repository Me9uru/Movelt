import { HubConnectionBuilder, HubConnectionState, LogLevel } from "@microsoft/signalr";
import { MessagePackHubProtocol } from "@microsoft/signalr-protocol-msgpack";

import { apiBaseUrl, getSessionToken } from "./auth";

export class LightNovelError extends Error {
  constructor(message: string, public readonly status = 500) {
    super(message);
    this.name = "LightNovelError";
  }
}

type Envelope<T> = { Success: boolean; Response: T; Status: number; Msg: string };
let hub: ReturnType<typeof createHub> | null = null;
let startPromise: Promise<void> | null = null;
let requestQueue = Promise.resolve();

function createHub() {
  return new HubConnectionBuilder()
    .withUrl(`${apiBaseUrl()}/hub/api`, { accessTokenFactory: getSessionToken })
    .withAutomaticReconnect({ nextRetryDelayInMilliseconds: ({ previousRetryCount }) => [0, 5_000, 10_000, 20_000][previousRetryCount] ?? 30_000 })
    .withHubProtocol(new MessagePackHubProtocol())
    .configureLogging(LogLevel.Error)
    .build();
}

async function connection() {
  if (!hub) hub = createHub();
  if (hub.state === HubConnectionState.Connected) return hub;
  if (!startPromise) {
    startPromise = hub.start().finally(() => { startPromise = null; });
  }
  await startPromise;
  return hub;
}

/** Direct equivalent of the official Web client's SignalR invocation wrapper. */
export async function requestWithHub<T>(method: string, payload: object = {}): Promise<T> {
  const previous = requestQueue;
  let release!: () => void;
  requestQueue = new Promise<void>((resolve) => { release = resolve; });
  await previous;
  try {
    const result = await (await connection()).invoke<Envelope<T>>(method, payload, { UseGzip: false });
    if (!result.Success) throw new LightNovelError(result.Msg, result.Status);
    return result.Response;
  } finally {
    release();
  }
}

export async function requestWithFetch<T>(path: string, payload: object, xId: string): Promise<T> {
  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method: "POST",
    headers: { Accept: "application/json", "Content-Type": "application/json", "x-id": xId },
    body: JSON.stringify(payload),
  });
  const value = await response.json() as Envelope<T>;
  if (!response.ok || !value.Success) throw new LightNovelError(value.Msg || `HTTP ${response.status}`, value.Status || response.status);
  return value.Response;
}

export async function rebootHub(): Promise<void> {
  if (!hub) return;
  const current = hub;
  hub = null;
  startPromise = null;
  await current.stop();
}
