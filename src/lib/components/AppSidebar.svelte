<script lang="ts">
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { type ComponentProps } from "svelte";
  import { Users, Settings, Folder, LayoutDashboard, Briefcase } from "lucide-svelte";
  import { sandboxes } from '$lib/stores/sandboxStatus';
  import Separator from "./ui/separator/separator.svelte";

  let sidebar = Sidebar.useSidebar();
  sidebar.setOpen(false);

  let {
    ref = $bindable(null),
    collapsible = "icon",
    ...restProps
  }: ComponentProps<typeof Sidebar.Root> = $props();

</script>

<Sidebar.Root class="mt-[36px]" {collapsible} {...restProps} >
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>
        <span>Overview</span>
      </Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton>
              {#snippet child({ props })}
                <a {...props} href="/">
                  <LayoutDashboard class="size-4" />
                  <span>Dashboard</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton>
              {#snippet child({ props })}
                <a {...props} href="/projects">
                  <Folder class="size-4" />
                  <span>Projects</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton>
              {#snippet child({ props })}
                <a {...props} href="/users">
                    <Users size={20} />
                    <span>Users</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton>
              {#snippet child({ props })}
                <a {...props} href="/businesses">
                    <Briefcase size={20} />
                    <span>Businesses</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton>
              {#snippet child({ props })}
                <a {...props} href="/settings" >
                    <Settings size={20} />
                    <span>Settings</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
          <Separator class="mt-8"/>
          {#each $sandboxes as info}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton>
                {#snippet child({ props })}
                  <a {...props} href="/projects/{info.project_id}" >
                    <div class="min-w-[16px] animate-pulse size-[16px] rounded-full" class:bg-green-700={info.status == "on"} class:bg-red-500={info.status == "off"} ></div>  
                    <span>{info.name} <b>{info.port}</b></span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
  <Sidebar.Footer>
  </Sidebar.Footer>
  <Sidebar.Rail />
</Sidebar.Root>
