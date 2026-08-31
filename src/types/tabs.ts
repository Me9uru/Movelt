export interface AppSectionHeaderProps<T extends string> {
  tabs: readonly { readonly name: T; readonly label: string }[];
  modelValue: T;
  swipe?: boolean;
}
