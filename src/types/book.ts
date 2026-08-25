/** Generic presentation models consumed by reusable book components. */
export interface BookGridItem<T> {
  id: string;
  title: string;
  coverUrl: string | null;
  data: T;
}

export interface BookChapterItem<T> {
  id: string;
  title: string;
  data: T;
  /** Leading icon rendered when the list is not paginated. */
  icon?: string;
  /** Secondary line below the chapter title. */
  meta?: string;
  unread?: boolean;
}

export interface BookChapterGroup<T> {
  title: string;
  count: number;
  chapters: BookChapterItem<T>[];
  groups?: BookChapterGroup<T>[];
}
