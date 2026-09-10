/** Ignore responses superseded by a newer request or a disposed owner. */
export const createLatestRequest = () => {
  let version = 0;
  return {
    start: () => {
      const request = ++version;
      return () => request === version;
    },
    invalidate: () => { version += 1; },
  };
};
