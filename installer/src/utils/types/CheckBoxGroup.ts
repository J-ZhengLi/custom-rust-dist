import { Component } from 'vue';

export interface CheckItem<T> {
  label: string;
  checked: boolean;
  required?: boolean;
  disabled?: boolean;
  value: T;
}

export interface CheckGroupItem<T> extends CheckItem<T> {
  focused: boolean;
  labelComponent?: Component;
  labelComponentProps?: Object;
}

export interface CheckGroup<T> {
  label: string;
  items: CheckGroupItem<T>[];
}
