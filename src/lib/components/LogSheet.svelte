<script lang="ts">
  import { Sheet, SheetContent, SheetHeader, SheetTitle } from "$lib/components/ui/sheet";
  import { Badge } from "$lib/components/ui/badge";
  import { Separator } from "$lib/components/ui/separator";
  import { Calendar, Globe, Database, Info } from "lucide-svelte";
  import {type ApiLog} from '$lib/api'
  import SvelteMarkdown from "svelte-markdown";
  import errorHelp from '$lib/errorhelp'
  

  // Parsed request/response interfaces
  interface ParsedRequestResponse {
    headers?: Record<string, string>;
    body?: any;
  }

  export let log: ApiLog;
  export let open: boolean = false; // Allow external control

  const formatTimestamp = (date: string) => {
    return new Date(date).toLocaleString(undefined, {
      dateStyle: "medium",
      timeStyle: "short"
    });
  };

  const parseJsonSafely = (jsonString?: string): ParsedRequestResponse | null => {
    if (!jsonString) return null;
    try {
      return JSON.parse(jsonString);
    } catch (e) {
      console.warn('Failed to parse JSON:', e);
      return null;
    }
  };

  $: parsedRequest = parseJsonSafely(log.request_body);
  $: parsedResponse = parseJsonSafely(log.response_body);

  const getStatusColor = (status: number) => {
    if (status >= 200 && status < 300) return "text-green-600";
    if (status >= 300 && status < 400) return "text-yellow-600";
    if (status >= 400 && status < 500) return "text-orange-600";
    if (status >= 500) return "text-red-600";
    return "text-gray-600";
  };

  const getMethodColor = (method: string) => {
    switch (method.toLowerCase()) {
      case 'get': return "bg-blue-100 text-blue-800";
      case 'post': return "bg-green-100 text-green-800";
      case 'put': return "bg-yellow-100 text-yellow-800";
      case 'patch': return "bg-purple-100 text-purple-800";
      case 'delete': return "bg-red-100 text-red-800";
      default: return "bg-gray-100 text-gray-800";
    }
  };

  const formatBody = (body: any): string => {
    if (typeof body === "string") {
      try {
        // Try to parse and re-stringify for better formatting
        const parsed = JSON.parse(body);
        return JSON.stringify(parsed, null, 2);
      } catch {
        return body;
      }
    }
    return JSON.stringify(body, null, 2);
  };

  // Function to open sheet externally
  export const openSheet = () => {
    open = true;
  };
</script>

<Sheet bind:open>
  <SheetContent class="max-w-[800px] mt-[36px] p-8 pb-16 w-full overflow-y-auto">
    <SheetHeader>
      <SheetTitle class="flex items-center gap-2 text-left">
        <Badge variant="outline" class={getMethodColor(log.method)}>
          {log.method}
        </Badge>
        <span class="font-mono text-sm text-blue-600">{log.path}</span>
        <Badge variant="outline" class={getStatusColor(log.status_code)}>
          {log.status_code}
        </Badge>
      </SheetTitle>
    </SheetHeader>

    <div class="mt-6 space-y-6">
      <!-- Metadata Section -->
      <div class="space-y-3">
        <div class="flex flex-col gap-2 text-sm text-muted-foreground">
          <span class="flex gap-2 font-bold items-center"><Calendar class="w-4 h-4" /> Timestamp: </span>
          <span> {formatTimestamp(log.created_at)}</span>
        </div>
        <div class="flex flex-col gap-2 text-sm text-muted-foreground">
          <span class="flex gap-2 font-bold items-center"><Database class="w-4 h-4" />Request ID: </span>
          <span> {log.id}</span>
        </div>
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <span class="flex gap-2 font-bold items-center"><Globe class="w-4 h-4" />Project ID: </span>
          <span> {log.project_id}</span>
        </div>
        {#if log.error_desc}
          <div class="flex flex-col gap-2 text-sm text-muted-foreground">
            <span class="flex gap-2 font-bold items-center"><Info class="w-4 h-4" />Help: </span>
            <span>
              <SvelteMarkdown source={errorHelp[log.error_desc] || log.error_desc} />
            </span>
          </div>
        {/if}
      </div>

      <Separator />

      <!-- Request Section -->
      <div class="space-y-4">
        <h3 class="text-lg font-semibold flex items-center gap-2">
          <span class="text-blue-600">→</span> Request
        </h3>
        
        {#if parsedRequest?.headers && Object.keys(parsedRequest.headers).length > 0}
          <div class="space-y-2">
            <h4 class="text-sm font-medium text-muted-foreground">Headers</h4>
            <div class="bg-muted/50 rounded-lg p-3">
              {#each Object.entries(parsedRequest.headers) as [key, value]}
                <div class="flex flex-wrap gap-1 py-1 text-sm">
                  <code class="bg-background px-2 py-0.5 rounded text-xs font-mono">{key}:</code>
                  <span class="break-all">{value}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        {#if parsedRequest?.body}
          <div class="space-y-2">
            <h4 class="text-sm font-medium text-muted-foreground">Body</h4>
            <pre class="bg-muted/50 rounded-lg p-3 text-xs font-mono overflow-auto max-h-64 whitespace-pre-wrap">{formatBody(parsedRequest.body)}</pre>
          </div>
        {:else}
          <p class="text-sm text-muted-foreground italic">No request body</p>
        {/if}
      </div>

      <Separator />

      <!-- Response Section -->
      <div class="space-y-4">
        <h3 class="text-lg font-semibold flex items-center gap-2">
          <span class="text-green-600">←</span> Response
        </h3>
        
        {#if parsedResponse?.headers && Object.keys(parsedResponse.headers).length > 0}
          <div class="space-y-2">
            <h4 class="text-sm font-medium text-muted-foreground">Headers</h4>
            <div class="bg-muted/50 rounded-lg p-3">
              {#each Object.entries(parsedResponse.headers) as [key, value]}
                <div class="flex flex-wrap gap-1 py-1 text-sm">
                  <code class="bg-background px-2 py-0.5 rounded text-xs font-mono">{key}:</code>
                  <span class="break-all">{value}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        {#if parsedResponse?.body}
          <div class="space-y-2">
            <h4 class="text-sm font-medium text-muted-foreground">Body</h4>
            <pre class="bg-muted/50 rounded-lg p-3 text-xs font-mono overflow-auto max-h-64 whitespace-pre-wrap">{formatBody(parsedResponse.body)}</pre>
          </div>
        {:else}
          <p class="text-sm text-muted-foreground italic">No response body</p>
        {/if}
      </div>
    </div>
  </SheetContent>
</Sheet>
