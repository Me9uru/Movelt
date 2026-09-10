const normalizeSearchText = (value: string): string =>
  value
    .normalize("NFKC")
    .toLocaleLowerCase()
    .replace(/[\p{P}\p{S}\s]+/gu, "");

/** Matches query characters in order while allowing gaps in the target text. */
export const fuzzyIncludes = (value: string, query: string): boolean => {
  const normalizedValue = normalizeSearchText(value);
  const normalizedQuery = normalizeSearchText(query);
  if (!normalizedQuery) return true;
  if (normalizedValue.includes(normalizedQuery)) return true;

  const queryCharacters = Array.from(normalizedQuery);
  let queryIndex = 0;
  for (const character of normalizedValue) {
    if (character !== queryCharacters[queryIndex]) continue;
    queryIndex += 1;
    if (queryIndex === queryCharacters.length) return true;
  }

  return false;
};
