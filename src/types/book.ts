/** Generic presentation models consumed by reusable book components. */
export interface BookGridItem<T> {
  id: string;
  title: string;
  coverUrl: string | null;
  meta?: string | null;
  coverStatus?: "finished" | "unfinished";
  data: T;
}

export interface BookCollectionState {
  icon?: string;
  title: string;
  description?: string;
}

export interface BookChapterItem<T> {
  id: string;
  title: string;
  data: T;
  icon?: string;
  meta?: string;
  unread?: boolean;
}

export interface BookChapterGroup<T> {
  title: string;
  count: number;
  chapters: BookChapterItem<T>[];
  groups?: BookChapterGroup<T>[];
}
