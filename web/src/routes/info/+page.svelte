<script lang="ts">
  import Separator from "$lib/components/ui/separator/separator.svelte";
  import * as Card from "$lib/components/ui/card";
  import { Github, Heart } from "lucide-svelte";
  import { onMount } from "svelte";
  import { getAppInfo, type AppInfo } from "$lib/api";

  let appInfo: AppInfo = {
    authors: "",
    description: "",
    name: "",
    version: "",
  };

  onMount(() => {
    getAppInfo().then((info) => {
      appInfo = info;
    });
  });
</script>

<div class="p-6 flex flex-col h-full">
  <div class="text-center my-8">
    <img
      src="/pesaplayground_logo.png"
      alt="Pesa Playground Logo"
      class="max-w-sm mx-auto mb-2"
    />
    <span class="text-xs text-muted-foreground">
      v{appInfo.version}
    </span>
  </div>

  <div class="prose dark:prose-invert max-w-none mx-auto text-center">
    <h1>Welcome to Pesa Playground</h1>
    <p class="lead">
      {appInfo.description}
    </p>
    <p>
      Pesa Playground provides a complete local simulation of the M-Pesa
      ecosystem, allowing you to test payment flows, STK push interactions, and
      API responses without needing to connect to external services. Use the
      sidebar to explore the documentation for specific APIs and features.
    </p>
  </div>

  <Separator class="my-8" />

  <div class="grid grid-cols-1 w-full md:grid-cols-2 gap-6 max-w-4xl mx-auto">
    <Card.Root class="hover:bg-muted/50 transition-colors">
      <a
        href="https://github.com/OmentaElvis/pesa-playground"
        target="_blank"
        rel="noopener noreferrer"
        class="p-6 block"
      >
        <Card.Header class="p-0">
          <Github class="h-8 w-8 mb-4" />
          <Card.Title>Contribute on GitHub</Card.Title>
          <Card.Description class="mt-2"
            >Found a bug or have an idea? The project is open-source.
            Contributions are welcome!</Card.Description
          >
        </Card.Header>
      </a>
    </Card.Root>
    <Card.Root class="hover:bg-muted/50 transition-colors">
      <a
        href="https://ko-fi.com/omenta"
        target="_blank"
        rel="noopener noreferrer"
        class="p-6 block"
      >
        <Card.Header class="p-0">
          <img
            src="https://storage.ko-fi.com/cdn/cup-border.png"
            alt="Ko-fi"
            class="h-8 w-auto mb-4"
          />
          <Card.Title>Support the Project</Card.Title>
          <Card.Description class="mt-2"
            >If Pesa Playground helps your workflow, consider supporting its
            development with a coffee.</Card.Description
          >
        </Card.Header>
      </a>
    </Card.Root>
  </div>

  <div class="flex-grow"></div>

  <footer class="text-center py-6 text-muted-foreground text-sm">
    <p class="flex items-center justify-center gap-1.5">
      Made with <Heart class="h-4 w-4 text-red-500" /> by {appInfo.authors}
    </p>
  </footer>
</div>
