<script lang="ts">
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import * as Card from '$lib/components/ui/card';
	import { Github, Heart } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { getAppInfo, type AppInfo } from '$lib/api';

	let appInfo: AppInfo = {
		authors: '',
		description: '',
		name: '',
		version: ''
	};

	onMount(() => {
		getAppInfo().then((info) => {
			appInfo = info;
		});
	});
</script>

<div class="flex h-full flex-col p-6">
	<div class="my-8 text-center">
		<img src="/pesaplayground_logo.png" alt="Pesa Playground Logo" class="mx-auto mb-2 max-w-sm" />
		<span class="text-xs text-muted-foreground">
			v{appInfo.version}
		</span>
	</div>

	<div class="mx-auto prose max-w-none text-center dark:prose-invert">
		<h1>Welcome to Pesa Playground</h1>
		<p class="lead">
			{appInfo.description}
		</p>
		<p>
			Pesa Playground provides a complete local simulation of the M-Pesa ecosystem, allowing you to
			test payment flows, STK push interactions, and API responses without needing to connect to
			external services. Use the sidebar to explore the documentation for specific APIs and
			features.
		</p>
	</div>

	<Separator class="my-8" />

	<div class="mx-auto grid w-full max-w-4xl grid-cols-1 gap-6 md:grid-cols-3">
		<Card.Root class="transition-colors hover:bg-muted/50">
			<a
				href="https://github.com/OmentaElvis/pesa-playground"
				target="_blank"
				rel="noopener noreferrer"
				class="block p-6"
			>
				<Card.Header class="p-0">
					<Github class="mb-4 h-8 w-8" />
					<Card.Title>Contribute on GitHub</Card.Title>
					<Card.Description class="mt-2">
						Found a bug or have an idea? The project is open-source. Contributions are welcome!
					</Card.Description>
				</Card.Header>
			</a>
		</Card.Root>
		<Card.Root class="transition-colors hover:bg-muted/50">
			<a
				href="https://discord.gg/jSbVJbTV6J"
				target="_blank"
				rel="noopener noreferrer"
				class="block p-6"
			>
				<Card.Header class="p-0">
					<img
						src="https://pngimg.com/uploads/discord/discord_PNG7.png"
						alt="Discord"
						class="mb-4 h-8 w-auto"
					/>
					<Card.Title>Join our Discord</Card.Title>
					<Card.Description class="mt-2">
						Connect with other developers and get support.
					</Card.Description>
				</Card.Header>
			</a>
		</Card.Root>
		<Card.Root class="transition-colors hover:bg-muted/50">
			<a
				href="https://ko-fi.com/omenta"
				target="_blank"
				rel="noopener noreferrer"
				class="block p-6"
			>
				<Card.Header class="p-0">
					<img
						src="https://storage.ko-fi.com/cdn/cup-border.png"
						alt="Ko-fi"
						class="mb-4 h-8 w-auto"
					/>
					<Card.Title>Support the Project</Card.Title>
					<Card.Description class="mt-2">
						If Pesa Playground helps your workflow, consider supporting its development with a
						coffee.
					</Card.Description>
				</Card.Header>
			</a>
		</Card.Root>
	</div>

	<div class="flex-grow"></div>

	<footer class="py-6 text-center text-sm text-muted-foreground">
		<p class="flex items-center justify-center gap-1.5">
			Made with <Heart class="h-4 w-4 text-red-500" /> by {appInfo.authors}
		</p>
	</footer>
</div>
