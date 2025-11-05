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
  const pendingWidgetId = writable<string | null>(null);

  const register = (widget: WidgetConfig) => {
    widgets.update(ws => {
      ws.set(widget.id, widget);
      lastAddedWidgetId.set(widget.id);
      return new Map(ws);
    });

    // Check if the newly registered widget was the one we were waiting for
    if (get(pendingWidgetId) === widget.id) {
      setActiveWidget(widget.id);
      pendingWidgetId.set(null); // Clear the pending state
    }
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
    if (get(activeWidgetId) === id && !get(isCollapsed)) {
        isCollapsed.set(true);
    } else {
        activeWidgetId.set(id);
        isCollapsed.set(false);
    }
    pendingWidgetId.set(null); // An explicit action clears any pending state
  };

  const collapse = () => {
    isCollapsed.set(true);
  };

  const acknowledgeNewWidget = () => {
    lastAddedWidgetId.set(null);
  };

  const initFromUrl = (params: URLSearchParams) => {
    const widgetId = params.get('widget');
    const collapsed = params.get('collapsed');

    if (widgetId) {
      // Don't set active directly, set as pending
      pendingWidgetId.set(widgetId);
    }

    if (collapsed === 'true') {
      isCollapsed.set(true);
    }
  };

  const resolvePending = () => {
    // If after a timeout there's still a pending widget, it means it was invalid.
    // So we clear the pending state and collapse.
    if (get(pendingWidgetId)) {
      pendingWidgetId.set(null);
      collapse();
    }
  };

  const widgetArray = derived(widgets, $widgets => Array.from($widgets.values()));
  const activeWidget = derived([widgets, activeWidgetId], ([$widgets, $activeWidgetId]) => 
    $activeWidgetId ? $widgets.get($activeWidgetId) : null
  );

  return {
    subscribe: isCollapsed.subscribe,
    widgets: widgetArray,
    activeWidget,
    lastAddedWidgetId,
    isCollapsed,
    register,
    unregister,
    setActiveWidget,
    collapse,
    acknowledgeNewWidget,
    initFromUrl,
    resolvePending,
  };
}

export const sidebarStore = createSidebarStore();