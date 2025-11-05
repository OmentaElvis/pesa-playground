import { writable, derived, get } from 'svelte/store';

export interface WidgetConfig {
  id: string;
  title: string;
  icon: any;
  component: any;
  props?: Record<string, any>;
}

function createSidebarStore() {
  const widgets = writable<Map<string, WidgetConfig>>(new Map());
  const activeWidgetId = writable<string | null>(null);
  const isCollapsed = writable<boolean>(true);
  const lastAddedWidgetId = writable<string | null>(null);

  const register = (widget: WidgetConfig) => {
    widgets.update(ws => {
      ws.set(widget.id, widget);
      lastAddedWidgetId.set(widget.id);
      return new Map(ws);
    });
  };

  const unregister = (id: string) => {
    widgets.update(ws => {
      ws.delete(id);

      if (get(activeWidgetId) === id) {
        let map = get(widgets);
        if (map.size == 0) {
          isCollapsed.set(true);
        } else {
          let keys = map.keys();
          activeWidgetId.set(keys.next().value || null);
        }
      }
      
      return new Map(ws);
    });
  };

  const setActiveWidget = (id: string) => {
    if (get(activeWidgetId) === id) {
      isCollapsed.update((value)=> {
        return !value;
      });

      return;
    }
    activeWidgetId.set(id);
    isCollapsed.set(false);
  };

  const collapse = () => {
    isCollapsed.set(true);
  };

  const acknowledgeNewWidget = () => {
    lastAddedWidgetId.set(null);
  };

  const widgetArray = derived(widgets, $widgets => Array.from($widgets.values()));
  const activeWidget = derived([widgets, activeWidgetId], ([$widgets, $activeWidgetId]) => 
    $activeWidgetId ? $widgets.get($activeWidgetId) : null
  );

  return {
    subscribe: isCollapsed.subscribe, // Default subscription is to isCollapsed
    widgets: widgetArray, // Expose widgets as a derived array store
    activeWidget, // Expose the active widget object
    lastAddedWidgetId,
    isCollapsed,
    register,
    unregister,
    setActiveWidget,
    collapse,
    acknowledgeNewWidget,
  };
}

export const sidebarStore = createSidebarStore();
