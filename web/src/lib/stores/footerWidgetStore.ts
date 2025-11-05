
import { writable, derived } from 'svelte/store';

export interface FooterWidgetConfig {
  id: string;
  component: any;
  props?: Record<string, any>;
}

function createFooterWidgetStore() {
  const widgets = writable<Map<string, FooterWidgetConfig>>(new Map());

  const register = (widget: FooterWidgetConfig) => {
    widgets.update(ws => {
      ws.set(widget.id, widget);
      return new Map(ws);
    });
  };

  const unregister = (id: string) => {
    widgets.update(ws => {
      ws.delete(id);
      return new Map(ws);
    });
  };

  const widgetArray = derived(widgets, $widgets => Array.from($widgets.values()));

  return {
    subscribe: widgetArray.subscribe,
    register,
    unregister,
  };
}

export const footerWidgetStore = createFooterWidgetStore();
