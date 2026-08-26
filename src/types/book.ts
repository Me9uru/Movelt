/** Generic presentation models consumed by reusable book components. */
export interface BookGridItem<T> {
  id: string;
  title: string;
  coverUrl: string | null;
  meta?: string | null;
  data: T;
}

export interface BookCollectionPagination {
  page: number;
  last: number;
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

/** Summary metric displayed beside a book's author and status. */
export interface BookDetailStat {
  value?: string | number;
  label: string;
}

/** Presentation contract shared by the novel and manga detail layouts. */
export interface BookDetailLayoutProps {
  title: string;
  coverUrl: string | null;
  author: string;
  status?: string | null;
  tags: string[];
  description: string | null | undefined;
  descriptionFallback?: string;
  onBookshelf: boolean;
  loading: boolean;
  resumeChapterId?: string | null;
  canStartReading?: boolean;
  stats?: BookDetailStat[];
  sectionTitle: string;
  sectionSummary: string;
}

/** Props accepted by the reusable book grid. */
export interface BookGridProps<T> {
  items: BookGridItem<T>[];
  disabled?: boolean;
}

/** Props accepted by a book collection with loading, error, and pagination states. */
export interface BookCollectionProps<T> {
  items: BookGridItem<T>[] | null;
  loading: boolean;
  error?: string;
  pagination?: BookCollectionPagination | null;
  disabled?: boolean;
  promptMessage?: string;
  emptyMessage?: string;
  errorTitle?: string;
  gridClass?: string;
  hasContent?: boolean;
  contentWhileLoading?: boolean;
}

/** Props accepted by the shared chapter list used by book detail pages. */
export interface BookChapterListProps<T> {
  items?: BookChapterItem<T>[];
  groups?: BookChapterGroup<T>[];
  loading?: boolean;
  pageSize?: number;
}
