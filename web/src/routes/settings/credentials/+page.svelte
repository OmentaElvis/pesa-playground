<script lang="ts">
	import { generateSecurityCredential } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import * as Field from '$lib/components/ui/field/index.js';
	import * as InputGroup from '$lib/components/ui/input-group/index.js';
	import { copyToClipboard } from '$lib/utils';
	import { Copy, Eye, EyeClosed, Hammer, KeyIcon } from 'lucide-svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import { settings } from '$lib/stores/settings';

	let showPassword = $state(false);
	let error: string | null = $state(null);
	let generated: string | null = $state(null);

	function togglePassword() {
		if (showPassword) {
			showPassword = false;
		} else {
			showPassword = true;
		}
	}

	async function generate(value: string) {
		try {
			generated = await generateSecurityCredential(value);
		} catch (err) {
			error = `${err}`;
		}
	}

	async function onsubmit(e: Event) {
		generated = null;
		// @ts-ignore
		let value = e.target.password.value;
		generate(value);
	}
</script>

<main class="container mx-auto space-y-6 p-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight text-foreground">
				Generate B2C Security Credential
			</h1>
			<p class="mt-1 text-muted-foreground">
				Generate an encrypted security credential from your initiator password. This credential is
				required for Business-to-Customer (B2C) API calls.
			</p>
		</div>
	</div>
	<Tabs.Root value="generate">
		<Tabs.List>
			<Tabs.Trigger value="generate"><Hammer /> Generate</Tabs.Trigger>
			<Tabs.Trigger value="public_key"><KeyIcon /> Public key</Tabs.Trigger>
		</Tabs.List>
		<Tabs.Content value="generate">
			<div>
				<form {onsubmit}>
					<Field.Group>
						<Field.Field>
							<Field.Label for="password">Initiator Password</Field.Label>
							<Field.Description>
								Enter the password that will be encrypted to generate your B2C security credential.
							</Field.Description>
							<InputGroup.Root>
								<InputGroup.Input
									id="password"
									required
									type={showPassword ? 'text' : 'password'}
								/>
								<InputGroup.Addon align="inline-end">
									<Button class="cursor-pointer" variant="ghost" onclick={togglePassword}>
										{#if !showPassword}
											<Eye />
										{:else}
											<EyeClosed />
										{/if}
									</Button>
								</InputGroup.Addon>
							</InputGroup.Root>
						</Field.Field>
						<Field.Field>
							<div>
								<Button type="submit">Generate</Button>
							</div>
						</Field.Field>
					</Field.Group>
				</form>
				<div class="text-red-500">
					{#if error}
						{error}
					{/if}
				</div>
				<div class="mt-4">
					{#if generated}
						<Field.Field>
							<Field.Label>Generated B2C Security Credential</Field.Label>
							<InputGroup.Root>
								<InputGroup.Textarea rows={5} value={generated} readonly />
								<InputGroup.Addon align="inline-end">
									<Button
										variant="outline"
										class="cursor-pointer"
										onclick={() => copyToClipboard(generated || '')}
									>
										<Copy />
									</Button>
								</InputGroup.Addon>
							</InputGroup.Root>
						</Field.Field>
					{/if}
				</div>
			</div>
		</Tabs.Content>
		<Tabs.Content value="public_key">
			<Field.Group>
				<Field.Field>
					<Field.Label for="public_key_display">System Public Key</Field.Label>
					<Field.Description>
						This is the public key currently used by the system. Note that this is a
						sandbox-specific key and is not the same as the public key provided by Safaricom for
						production environments.
					</Field.Description>
					<InputGroup.Root>
						<InputGroup.Textarea rows={10} value={$settings.encryption_keys?.public_key} readonly />
						<InputGroup.Addon align="inline-end">
							<Button
								variant="outline"
								class="cursor-pointer"
								onclick={() => copyToClipboard($settings.encryption_keys?.public_key || '')}
							>
								<Copy />
							</Button>
						</InputGroup.Addon>
					</InputGroup.Root>
				</Field.Field>
			</Field.Group>
		</Tabs.Content>
	</Tabs.Root>
</main>
