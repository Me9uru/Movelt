import { createRouter, createWebHashHistory } from "vue-router";

export type LibraryRouteName = "discovery" | "bookshelf";
export type AppRouteName = LibraryRouteName | "detail" | "reader";

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
    {
      path: "/library/:source/:bookId",
      name: "detail",
      component: () => import("../pages/library/NovelDetailPage.vue"),
    },
    {
      path: "/library/:source/:bookId/read/:chapterId",
      name: "reader",
      component: () => import("../pages/reader/ReaderPage.vue"),
    },
    { path: "/:pathMatch(.*)*", redirect: { name: "discovery" } },
  ],
  scrollBehavior: () => ({ top: 0 }),
});
