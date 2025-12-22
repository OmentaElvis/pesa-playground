<script lang="ts">
	import UserList from '$lib/components/users/UserList.svelte';
	import { PaneGroup, Pane } from 'paneforge';
	let { children } = $props();

	let minUserListWidth = 260;
	let containerWidth = $state(1);
	let userListPercent = $derived((minUserListWidth/containerWidth)*100);

</script>

<div class="flex h-full" bind:clientWidth={containerWidth} >
	<PaneGroup direction="horizontal" >
		<Pane defaultSize={userListPercent} >
			<UserList />
		</Pane>
		<Pane>
			<div class="h-full flex-1 overflow-y-auto">
				{@render children()}
			</div>
		</Pane>
	</PaneGroup>
</div>
