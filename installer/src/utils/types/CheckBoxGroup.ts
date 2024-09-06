export interface CheckItem<T> {
  label: string;
  checked: boolean;
  value: T;
}

export interface CheckGroupItem<T> extends CheckItem<T> {
  selected: boolean;
}

export interface CheckGroup<T> {
  label: string;
  items: CheckGroupItem<T>[];
}
