import { createRouter, createWebHashHistory } from "vue-router";

export type { AppRouteName, LibraryRouteName } from "../types/router";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: { name: "novels" } },
    {
      path: "/login",
      name: "login",
      component: () => import("../pages/auth/LoginPage.vue"),
    },
    {
      path: "/novels",
      name: "novels",
      component: () => import("../pages/novel/NovelDiscoveryPage.vue"),
    },
    {
      path: "/bookshelf",
      name: "bookshelf",
      component: () => import("../pages/bookshelf/BookshelfPage.vue"),
    },
    {
      path: "/comic",
      name: "comic",
      component: () => import("../pages/comic/ComicDiscoveryPage.vue"),
    },
    {
      path: "/comic/:comicId",
      name: "comic-detail",
      component: () => import("../pages/comic/ComicDetailPage.vue"),
    },
    {
      path: "/comic/:comicId/read/:chapterId",
      name: "comic-reader",
      component: () => import("../pages/comic/ComicReaderPage.vue"),
    },
    {
      path: "/library/:bookId",
      name: "detail",
      component: () => import("../pages/novel/NovelDetailPage.vue"),
    },
    {
      path: "/library/:bookId/read/:chapterId",
      name: "reader",
      component: () => import("../pages/novel/NovelReaderPage.vue"),
    },
    { path: "/:pathMatch(.*)*", redirect: { name: "novels" } },
  ],
  scrollBehavior: () => ({ top: 0 }),
});
