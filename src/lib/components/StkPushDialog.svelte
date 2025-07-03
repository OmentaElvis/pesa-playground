<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Label } from '$lib/components/ui/label';
  import { RadioGroup, RadioGroupItem } from '$lib/components/ui/radio-group';
  import { 
    Phone, 
    CheckCircle, 
    XCircle, 
    WifiOff, 
    Timer,
    X,
    Smartphone,
  } from 'lucide-svelte';

  export let dialogData: any = null;
  export let open = false;

  const dispatch = createEventDispatcher();

  let selectedAction = 'correct_pin';

  const actionOptions = [
    {
      id: 'correct_pin',
      label: 'Correct PIN Entry',
      description: 'User enters correct PIN and transaction succeeds',
      icon: CheckCircle,
      variant: 'default'
    },
    {
      id: 'wrong_pin',
      label: 'Wrong PIN Entry',
      description: 'User enters incorrect PIN and transaction fails',
      icon: XCircle,
      variant: 'destructive'
    },
    {
      id: 'user_offline',
      label: 'User Offline/Unreachable',
      description: 'User phone is offline or unreachable',
      icon: WifiOff,
      variant: 'secondary'
    },
    {
      id: 'timeout',
      label: 'Transaction Timeout',
      description: 'User does not respond within time limit',
      icon: Timer,
      variant: 'outline'
    },
    {
      id: 'cancel',
      label: 'User Cancels',
      description: 'User cancels the transaction on their phone',
      icon: X,
      variant: 'secondary'
    }
  ];

  function handleSubmit() {
    if (dialogData) {
      dispatch('action', {
        action: selectedAction,
        checkout_id: dialogData.checkout_id
      });
      
    }
    open = false;
  }

  function handleCancel() {
    if (dialogData) {
      dispatch('action',{
        action: selectedAction,
        checkout_id: dialogData.checkout_id
      });
    }
    open = false;
  }

  function formatPhoneNumber(phone: string) {
    if (!phone) return '';
    // Format as +254 759 289 552
    return phone.replace(/(\d{3})(\d{3})(\d{3})(\d{3})/, '+$1 $2 $3 $4');
  }

  function formatAmount(amount: number) {
    return new Intl.NumberFormat('en-KE', {
      style: 'currency',
      currency: 'KES',
      minimumFractionDigits: 0
    }).format(amount);
  }

  $: user = dialogData?.user;
  $: callback = dialogData?.callback;
</script>

<Dialog bind:open>
  <DialogContent class="max-w-4xl max-h-[90vh] overflow-y-auto">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <div>
          <div class="flex gap-2">
            <Smartphone class="h-5 w-5 text-primary" />
            STK Push
          </div>
          <div class="bg-muted px-2">
            <code class="text-xs font-mono font-light">{callback?.checkout_request_id}</code>
          </div>
        </div>
      </DialogTitle>
    </DialogHeader>

    <div class="space-y-6">
      <!-- Customer Information Card -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4">
        <div class="space-y-2">
          <Label class="text-sm font-medium text-muted-foreground">Name</Label>
          <p class="font-semibold">{user?.name}</p>
        </div>
        <div class="space-y-2">
          <Label class="text-sm font-medium text-muted-foreground">Phone Number</Label>
          <div class="flex items-center gap-2">
            <Phone class="h-4 w-4 text-green-600" />
            <span class="font-semibold">{formatPhoneNumber(user?.phone)}</span>
          </div>
        </div>
        <div class="space-y-2">
          <Label class="text-sm font-medium text-muted-foreground">M-Pesa Balance</Label>
          <p class="font-semibold text-green-600">{formatAmount(user?.balance)}</p>
        </div>
        <div class="space-y-2">
          <Label class="text-sm font-medium text-muted-foreground">Account Status</Label>
          <Badge variant="default" class="capitalize">{user?.status}</Badge>
        </div>
      </div>

      <!-- Action Selection Card -->
      <h2 class="text-lg">Response</h2>
      <RadioGroup bind:value={selectedAction} class="space-y-3 bg-muted p-4">
        {#each actionOptions as option}
          <div class="flex space-x-2">
            <RadioGroupItem value={option.id} id={option.id} />
            <Label for={option.id} class="flex-1 cursor-pointer">
              <div class="flex items-start gap-3 hover:bg-muted/50 transition-colors">
                <div class="mt-0.5">
                  <svelte:component this={option.icon} class="h-5 w-5 text-muted-foreground" />
                </div>
                <div class="flex-1">
                  <div class="font-medium">{option.label}</div>
                  <div class="text-sm text-muted-foreground mt-1">{option.description}</div>
                </div>
              </div>
            </Label>
          </div>
        {/each}
      </RadioGroup>
    </div>

    <!-- Footer Actions -->
    <div class="flex justify-end gap-2 pt-4 border-t">
      <Button variant="outline" onclick={handleCancel}>
        Cancel
      </Button>
      <Button onclick={handleSubmit} class="gap-2">
        <Smartphone class="h-4 w-4" />
        Simulate Response
      </Button>
    </div>
  </DialogContent>
</Dialog>
