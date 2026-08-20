import { createRouter, createWebHashHistory } from "vue-router";

export type LibraryRouteName = "novels" | "bookshelf" | "manga" | "settings";
export type AppRouteName = LibraryRouteName | "detail" | "reader" | "manga-detail" | "manga-reader";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: { name: "novels" } },
    {
      path: "/novels",
      name: "novels",
      component: () => import("../pages/library/DiscoveryPage.vue"),
    },
    {
      path: "/bookshelf",
      name: "bookshelf",
      component: () => import("../pages/library/BookshelfPage.vue"),
    },
    { path: "/manga", name: "manga", component: () => import("../pages/manga/MangaPage.vue") },
    { path: "/settings", name: "settings", component: () => import("../pages/settings/SettingsPage.vue") },
    { path: "/manga/:mangaId", name: "manga-detail", component: () => import("../pages/manga/MangaDetailPage.vue") },
    { path: "/manga/:mangaId/read/:chapterId", name: "manga-reader", component: () => import("../pages/manga/MangaReaderPage.vue") },
    {
      path: "/library/:bookId",
      name: "detail",
      component: () => import("../pages/library/NovelDetailPage.vue"),
    },
    {
      path: "/library/:bookId/read/:chapterId",
      name: "reader",
      component: () => import("../pages/reader/ReaderPage.vue"),
    },
    { path: "/:pathMatch(.*)*", redirect: { name: "novels" } },
  ],
  scrollBehavior: () => ({ top: 0 }),
});
