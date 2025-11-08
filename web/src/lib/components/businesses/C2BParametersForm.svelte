<script lang="ts">
  import * as Select from "$lib/components/ui/select";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { C2BResponseType } from "$lib/api";

  let {
    response_type = $bindable(),
    validation_url = $bindable(),
    confirmation_url = $bindable(),
  } = $props<{
    response_type: C2BResponseType | undefined;
    validation_url: string | undefined;
    confirmation_url: string | undefined;
  }>();
</script>

<span class="font-bold text-lg mt-4"> C2B parameters </span>
<span class="text-sm">
  These can be modified using c2b <pre>/mpesa/c2b/v1/registerurl</pre>
</span>
<div class="grid grid-cols-4 items-center gap-4">
  <Label for="responseType" class="text-right">Response Type</Label>
  <Select.Root type="single" bind:value={response_type}>
    <Select.Trigger class="col-span-3">
      {response_type || "Select a response type"}
    </Select.Trigger>
    <Select.Content>
      <Select.Item value={C2BResponseType.Completed}>Completed</Select.Item>
      <Select.Item value={C2BResponseType.Canceled}>Canceled</Select.Item>
    </Select.Content>
  </Select.Root>
</div>
<div class="grid grid-cols-4 items-center gap-4">
  <Label for="validationUrl" class="text-right">Validation URL</Label>
  <Input
    id="validationUrl"
    type="text"
    class="col-span-3"
    bind:value={validation_url}
  />
</div>
<div class="grid grid-cols-4 items-center gap-4">
  <Label for="confirmationUrl" class="text-right">Confirmation URL</Label>
  <Input
    id="confirmationUrl"
    type="text"
    class="col-span-3"
    bind:value={confirmation_url}
  />
</div>
