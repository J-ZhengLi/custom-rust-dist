interface CheckItem<T> {
  label: string;
  checked: boolean;
  value: T;
}

interface CheckGroupItem<T> extends CheckItem<T> {
  selected: boolean;
}

interface CheckGroup<T> {
  label: string;
  items: CheckGroupItem<T>[];
}
