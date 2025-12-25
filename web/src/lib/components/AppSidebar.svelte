<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { type ComponentProps } from 'svelte';
	import { Users, Settings, LayoutDashboard, Info, KeyIcon } from 'lucide-svelte';
	import { sandboxes } from '$lib/stores/sandboxStatus';
	import Separator from './ui/separator/separator.svelte';

	let sidebar = Sidebar.useSidebar();
	sidebar.setOpen(false);

	let {
		ref = $bindable(null),
		collapsible = 'icon',
		...restProps
	}: ComponentProps<typeof Sidebar.Root> = $props();
</script>

<Sidebar.Root class="mt-[36px]" {collapsible} {...restProps}>
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
								<a {...props} href="/projects">
									<LayoutDashboard class="size-4" />
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
								<a {...props} href="/settings/credentials">
									<KeyIcon size={20} />
									<span>Keys</span>
								</a>
							{/snippet}
						</Sidebar.MenuButton>
					</Sidebar.MenuItem>
					<Sidebar.MenuItem>
						<Sidebar.MenuButton>
							{#snippet child({ props })}
								<a {...props} href="/settings">
									<Settings size={20} />
									<span>Settings</span>
								</a>
							{/snippet}
						</Sidebar.MenuButton>
					</Sidebar.MenuItem>
					<Separator class="mt-8" />
					{#each $sandboxes as [_, info]}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props })}
									<a {...props} href="/projects/{info.project_id}">
										<div
											class="size-[16px] min-w-[16px] animate-pulse rounded-full"
											class:bg-green-700={info.status == 'on'}
											class:bg-orange-500={info.status == 'off'}
											class:bg-red-500={info.status == 'error'}
										></div>
										<span>
											{info.name}
											<b>{info.port}</b>
										</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>
	<Sidebar.Footer class="mb-[36px]">
		<Sidebar.MenuItem>
			<Sidebar.MenuButton>
				{#snippet child({ props })}
					<a {...props} href="/info">
						<Info size={20} />
						<span>Info</span>
					</a>
				{/snippet}
			</Sidebar.MenuButton>
		</Sidebar.MenuItem>
		<br />
	</Sidebar.Footer>
	<Sidebar.Rail />
</Sidebar.Root>
