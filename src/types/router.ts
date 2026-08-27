export type LibraryRouteName = "novels" | "bookshelf" | "comic" | "settings";
export type AppRouteName =
  | LibraryRouteName
  | "novel-search"
  | "login"
  | "detail"
  | "reader"
  | "comic-detail"
  | "comic-reader";

export type ReturnRouteName = LibraryRouteName | "novel-search";
