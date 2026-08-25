export type LibraryRouteName = "novels" | "bookshelf" | "manga" | "settings";
export type AppRouteName =
  | LibraryRouteName
  | "novel-search"
  | "login"
  | "detail"
  | "reader"
  | "manga-detail"
  | "manga-reader";

export type ReturnRouteName = LibraryRouteName | "novel-search";
