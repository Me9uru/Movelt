import { createRouter, createWebHashHistory } from "vue-router";

export type LibraryRouteName = "discovery" | "bookshelf" | "manga";
export type AppRouteName = LibraryRouteName | "detail" | "reader" | "manga-detail" | "manga-reader";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: { name: "discovery" } },
    {
      path: "/discovery",
      name: "discovery",
      component: () => import("../pages/library/DiscoveryPage.vue"),
    },
    {
      path: "/bookshelf",
      name: "bookshelf",
      component: () => import("../pages/library/BookshelfPage.vue"),
    },
    { path: "/manga", name: "manga", component: () => import("../pages/manga/MangaPage.vue") },
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
    { path: "/:pathMatch(.*)*", redirect: { name: "discovery" } },
  ],
  scrollBehavior: () => ({ top: 0 }),
});
