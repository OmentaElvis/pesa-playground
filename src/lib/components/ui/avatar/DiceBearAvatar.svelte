<script lang="ts">
  import { createAvatar } from "@dicebear/core";
  import { micah } from "@dicebear/collection";
  import * as Avatar from "$lib/components/ui/avatar/index.js";

  export let seed: string;
  export let fallback: string;

  const avatar = createAvatar(micah, {
    seed: seed,
    size: 128,
    radius: 50,
  });

  const avatarUri = avatar.toDataUri();
</script>

<Avatar.Root class="w-full h-full border-foreground bg-muted border bg-green-800">
  <div
    class="flex h-full w-full items-center justify-center overflow-hidden rounded-full border-2 border-transparent"
  >
    <Avatar.Image src={avatarUri} alt="User Avatar" />
    <Avatar.Fallback>{fallback}</Avatar.Fallback>
  </div>
</Avatar.Root>
