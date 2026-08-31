import { inject, provide, type InjectionKey } from "vue";

export type DiscoveryKind = "novel" | "comic";

export interface DiscoveryContext {
  readonly subjectLabel: string;
  readonly countUnit: string;
}

const discoveryContexts: Record<DiscoveryKind, DiscoveryContext> = {
  novel: { subjectLabel: "小说", countUnit: "本" },
  comic: { subjectLabel: "漫画", countUnit: "部" },
};

const defaultDiscoveryContext: DiscoveryContext = {
  subjectLabel: "作品",
  countUnit: "部",
};

const discoveryContextKey: InjectionKey<Readonly<DiscoveryContext>> = Symbol(
  "discovery-context",
);

/** Provides copy terminology only within the current discovery page subtree. */
export const provideDiscoveryContext = (kind: DiscoveryKind): void => {
  provide(discoveryContextKey, discoveryContexts[kind]);
};

export const useDiscoveryContext = (): Readonly<DiscoveryContext> => {
  return inject(discoveryContextKey, defaultDiscoveryContext);
};
