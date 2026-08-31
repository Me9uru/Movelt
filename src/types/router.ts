export type LibraryRouteName = "novels" | "bookshelf" | "comic";
export type AppRouteName =
  | LibraryRouteName
  | "login"
  | "detail"
  | "reader"
  | "comic-detail"
  | "comic-reader";

export type ReturnRouteName = LibraryRouteName;
