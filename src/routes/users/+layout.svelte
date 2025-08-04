<script lang="ts">
  import UserList from "$lib/components/users/UserList.svelte";
  import { PaneGroup, Pane} from 'paneforge'
	let { children } = $props();

</script>
<div class="flex h-full">
    <PaneGroup direction="horizontal">
        <Pane defaultSize={35}>
            <UserList />
        </Pane>
        <Pane>
            <div class="flex-1 overflow-y-auto h-full">
                {@render children()}
            </div>
        </Pane>
    </PaneGroup>
</div>
