<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import SandboxToggle from "$lib/components/SandboxToggle.svelte";
  import LogSheet from "$lib/components/LogSheet.svelte";

  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";
  import {
    Copy,
    Key,
    Activity,
    Users,
    Plus,
    Trash2,
    Settings,
    CheckCircle,
    XCircle,
    Clock,
    Globe,
    Code,
    AlertCircle,
    User,
    Phone,
    CreditCard,
    RefreshCw,
    LoaderCircle,
    CircleX,
    ChevronRight,
  } from "lucide-svelte";
  import {
    getProject,
    getUsers,
    removeUser,
    generateUser,
    createUser,
    listTransactions,
    countTransactions,
    sandboxStatus,
    type Transaction,
    type User as UserType,
    type ProjectDetails,
    type ApiLog,
    getProjectApiLogs,
  } from "$lib/api";
  import { page } from "$app/state";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  // Mock project data

  let id = $derived(page.params.id);
  let project: Promise<ProjectDetails> = $derived(getProject(Number(id)));
  let users: Promise<UserType[]> = $derived(getUsers(Number(id)));
  let apiLogs: Promise<ApiLog[]> = $derived(
    getProjectApiLogs({ project_id: Number(id), limit: 20 }),
  );

  let creatingUser = $state(false);

  let selectedLog: ApiLog | null = $state(null);
  let logSidebarOpen: boolean = $state(false);

  // New user form
  let newUser = $state({
    name: "",
    phone: "",
    balance: 10000,
    pin: "0000",
  });

  async function addUser() {
    creatingUser = true;
    try {
      await createUser({
        id: 0,
        name: newUser.name,
        phone: newUser.phone,
        balance: newUser.balance,
        project_id: Number(id),
        pin: newUser.pin,
      });
      newUser = {
        name: "",
        phone: "",
        balance: 10000,
        pin: "0000",
      };

      users = getUsers(Number(id));
    } catch (err) {
      console.log(err);
    } finally {
      creatingUser = false;
    }
  }

  function copyToClipboard(text: String = "") {
    navigator.clipboard.writeText(text as string);
    // Add toast notification here
  }

  function getStatusColor(status: number) {
    if (status >= 200 && status < 300) return "text-green-600";
    if (status >= 400 && status < 500) return "text-yellow-600";
    if (status >= 500) return "text-red-600";
    return "text-gray-600";
  }

  function getStatusIcon(status: number) {
    if (status >= 200 && status < 300) return CheckCircle;
    if (status >= 400) return XCircle;
    return AlertCircle;
  }

  async function generateRandomUser() {
    newUser = await generateUser();
  }

  async function deleteUser(user_id: number) {
    try {
      await removeUser(user_id);
    } catch (err) {
      console.log(err);
    } finally {
      users = getUsers(Number(id));
    }
  }
  function refreshLogs() {
    apiLogs = getProjectApiLogs({ project_id: Number(id), limit: 20 });
  }

  function debounce(func: Function, wait: number) {
    let timeout: number;
    return function (...args) {
      clearTimeout(timeout);
      timeout = setTimeout(() => func.apply(this, args), wait);
    };
  }

  const debouncedRefreshLogs = debounce(async () => {
    refreshLogs();
  }, 300);

  let unlisten: UnlistenFn;

  listen("new-api-log", (event) => {
    let project_id = event.payload;
    if (project_id == Number(id)) {
      debouncedRefreshLogs();
    }
  }).then((un) => {
    unlisten = un;
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });
</script>

<main class="container mx-auto p-6 space-y-6">
  <!-- Header -->
  {#await project}
    <div class="size-full flex items-center justify-center">
      <div>
        <LoaderCircle class="animate-spin" />
        <span>Loading items</span>
      </div>
    </div>
  {:then project}
    <div class="flex flex-col gap-4">
      <div class="flex justify-between items-center">
        <div>
          <div class="flex gap-2">
            <h1 class="text-3xl font-bold tracking-tight text-foreground">
              {project.name}
            </h1>
            <h3>
              #{project.id}
            </h3>
          </div>
          <div class="flex items-center gap-2 mt-2">
            <Badge
              class="bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300"
              variant="outline"
            >
              {project.simulation_mode}
            </Badge>
            <span class="text-muted-foreground">•</span>
            <span class="text-sm text-muted-foreground"
              >Shortcode: {project.shortcode}</span
            >
          </div>
        </div>
        <Button
          href={`/projects/${id}/settings`}
          variant="outline"
          class="gap-2"
        >
          <Settings class="h-4 w-4" />
          Settings
        </Button>
      </div>
      <div>
        <SandboxToggle id={Number(id)} />
      </div>
    </div>
    <!-- Stats Overview -->
    <div class="">
      <Card>
        <CardContent class="p-4 grid gap-4 md:grid-cols-4">
          <div class="flex items-center gap-2">
            <Activity class="text-muted-foreground" />
            <div>
              <p class="text-sm text-muted-foreground">Total Transactions</p>
              <p class="text-2xl font-bold">{0}</p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <Users class="text-muted-foreground" />
            <div>
              <p class="text-sm text-muted-foreground">Test Users</p>
              <p class="text-2xl font-bold">
                {#await users then userList}
                  {userList.length}
                {:catch _}
                  0
                {/await}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- Main Content Tabs -->
    <Tabs value="overview" class="w-full">
      <TabsList class="grid w-full grid-cols-4">
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="api-keys">API Keys</TabsTrigger>
        <TabsTrigger value="logs">API Logs</TabsTrigger>
        <TabsTrigger value="users">Test Users</TabsTrigger>
      </TabsList>

      <!-- Overview Tab -->
      <TabsContent value="overview" class="space-y-6">
        <div class="">
          <!-- Project Info -->
          <Card>
            <CardHeader>
              <CardTitle class="flex items-center gap-2">
                <Globe class="h-5 w-5" />
                Project Configuration
              </CardTitle>
            </CardHeader>
            <CardContent class="space-y-4">
              <div>
                <Label class="text-sm font-medium">Callback URL</Label>
                <div class="flex items-center gap-2 mt-1">
                  <Input value={project.callback_url} readonly class="flex-1" />
                  {#if project.callback_url}
                    <Button
                      size="sm"
                      variant="outline"
                      onclick={() => copyToClipboard(project.callback_url)}
                    >
                      <Copy class="h-4 w-4" />
                    </Button>
                  {/if}
                </div>
              </div>
              <div>
                <Label class="text-sm font-medium">Business Shortcode</Label>
                <div class="flex items-center gap-2 mt-1">
                  <Input value={project.shortcode} readonly class="flex-1" />
                  {#if project.shortcode}
                    <Button
                      size="sm"
                      variant="outline"
                      onclick={() => copyToClipboard(project.shortcode)}
                    >
                      <Copy class="h-4 w-4" />
                    </Button>
                  {/if}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </TabsContent>

      <!-- API Keys Tab -->
      <TabsContent value="api-keys" class="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Key class="h-5 w-5" />
              API Credentials
            </CardTitle>
            <p class="text-sm text-muted-foreground">
              Use these credentials to authenticate your API requests
            </p>
          </CardHeader>
          <CardContent class="space-y-6">
            <!-- API Key -->
            <div class="space-y-2">
              <Label class="text-sm font-medium">Client Key</Label>
              <div class="flex items-center gap-2">
                <Input
                  type="text"
                  value={project.consumer_key}
                  readonly
                  class="flex-1 font-mono"
                />
                <Button
                  size="sm"
                  variant="outline"
                  onclick={() => copyToClipboard(project.consumer_key)}
                >
                  <Copy class="h-4 w-4" />
                </Button>
              </div>
            </div>

            <!-- Secret Key -->
            <div class="space-y-2">
              <Label class="text-sm font-medium">Client Secret</Label>
              <div class="flex items-center gap-2">
                <Input
                  value={project.consumer_secret}
                  readonly
                  class="flex-1 font-mono"
                />
                <Button
                  size="sm"
                  variant="outline"
                  onclick={() => copyToClipboard(project.consumer_secret)}
                >
                  <Copy class="h-4 w-4" />
                </Button>
              </div>
            </div>
            <!-- PassKey -->
            <div class="space-y-2">
              <Label class="text-sm font-medium">PassKey</Label>
              <div class="flex items-center gap-2">
                <Input
                  value={project.passkey}
                  readonly
                  class="flex-1 font-mono"
                />
                <Button
                  size="sm"
                  variant="outline"
                  onclick={() => copyToClipboard(project.passkey)}
                >
                  <Copy class="h-4 w-4" />
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      </TabsContent>

      <!-- API Logs Tab -->
      <TabsContent value="logs" class="space-y-6">
        <Card>
          <CardHeader>
            <div class="flex justify-between items-center">
              <CardTitle class="flex items-center gap-2">
                <Activity class="h-5 w-5" />
                Recent API Activity
              </CardTitle>
              {#await apiLogs}
                <Button size="sm" variant="outline" disabled>
                  <RefreshCw class="animate-spin h-4 w-4 mr-2" />
                  Refresh
                </Button>
              {:then _}
                <Button size="sm" variant="outline" onclick={refreshLogs}>
                  <RefreshCw class="h-4 w-4 mr-2" />
                  Refresh
                </Button>
              {/await}
            </div>
          </CardHeader>
          <CardContent>
            <div class="space-y-4">
              {#await apiLogs}
                <div>
                  <LoaderCircle />
                </div>
              {:then apiLogs}
                {#each apiLogs as log (log.id)}
                  <div class="border rounded-lg p-4 space-y-2">
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-2">
                        <svelte:component
                          this={getStatusIcon(log.status_code)}
                          class="h-4 w-4 {getStatusColor(log.status_code)}"
                        />
                        <span class="font-mono text-sm font-medium"
                          >{log.method}</span
                        >
                        <span class="font-mono text-sm">{log.path}</span>
                        <Badge
                          variant="outline"
                          class={getStatusColor(log.status_code)}
                        >
                          {log.status_code}
                        </Badge>
                      </div>
                      <Button
                        variant="outline"
                        onclick={() => {
                          selectedLog = log;
                          logSidebarOpen = true;
                        }}
                      >
                        <ChevronRight />
                      </Button>
                    </div>
                    <span class="font-mono text-sm">{log.created_at}</span>
                  </div>
                {/each}
              {/await}
            </div>
          </CardContent>
        </Card>
      </TabsContent>

      <!-- Test Users Tab -->
      <TabsContent value="users" class="space-y-6">
        <!-- Add New User Card -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Plus class="h-5 w-5" />
              Add Test User
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="grid gap-4 md:grid-cols-4">
              <div>
                <Label for="user-name" class="text-sm font-medium">Name</Label>
                <Input
                  id="user-name"
                  bind:value={newUser.name}
                  placeholder="John Doe"
                />
              </div>
              <div>
                <Label for="user-phone" class="text-sm font-medium"
                  >Phone Number</Label
                >
                <Input
                  id="user-phone"
                  bind:value={newUser.phone}
                  placeholder="254712345678"
                />
              </div>
              <div>
                <Label for="user-balance" class="text-sm font-medium"
                  >Initial Balance (KES)</Label
                >
                <Input
                  id="user-balance"
                  type="number"
                  bind:value={newUser.balance}
                  min="0"
                />
              </div>
              <div>
                <Label for="pin" class="text-sm font-medium">Pin</Label>
                <Input id="pin" bind:value={newUser.pin} />
              </div>
              <div class="flex items-end gap-2">
                {#if !creatingUser}
                  <Button
                    onclick={addUser}
                    disabled={!newUser.name || !newUser.phone}
                  >
                    Add User
                  </Button>
                {:else}
                  <Button disabled>
                    <LoaderCircle class="animate-spin" />
                    Add User
                  </Button>
                {/if}
                <Button variant="outline" onclick={generateRandomUser}>
                  <RefreshCw class="h-4 w-4" />
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Users List -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Users class="h-5 w-5" />
              Test Users
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="space-y-4">
              {#await users then userList}
                {#each userList as user (user.id)}
                  <div class="border rounded-lg p-4">
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-4">
                        <div
                          class="w-10 h-10 bg-primary/10 rounded-full flex items-center justify-center"
                        >
                          <User class="h-5 w-5 text-primary" />
                        </div>
                        <div>
                          <h4 class="font-medium">{user.name}</h4>
                          <div
                            class="flex items-center gap-4 text-sm text-muted-foreground"
                          >
                            <span class="flex items-center gap-1">
                              <Phone class="h-3 w-3" />
                              {user.phone}
                            </span>
                            <span class="flex items-center gap-1">
                              <CreditCard class="h-3 w-3" />
                              KES {user.balance.toLocaleString()}
                            </span>
                            <span class="flex items-center gap-1">
                              <Clock class="h-3 w-3" />
                              0
                            </span>
                          </div>
                        </div>
                      </div>
                      <div class="flex items-center gap-2">
                        <Badge
                          class={user.status === "active"
                            ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300"
                            : "bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300"}
                        >
                          {user.status}
                        </Badge>
                        <Button
                          size="sm"
                          variant="ghost"
                          onclick={() => deleteUser(user.id)}
                          class="text-destructive hover:text-destructive"
                        >
                          <Trash2 class="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                  </div>
                {:else}
                  <div class="text-center py-8 text-muted-foreground">
                    <Users class="h-12 w-12 mx-auto mb-4 opacity-50" />
                    <p>No test users yet. Add your first user above.</p>
                  </div>
                {/each}
              {:catch reason}
                <div class="text-center py-8 text-muted-foreground">
                  <CircleX class="h-12 w-12 mx-auto mb-4 opacity-50" />
                  <p>Failed to load users: {reason}</p>
                </div>
              {/await}
            </div>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  {:catch reason}
    <span>Unable to load project: {reason}</span>
  {/await}

  {#if selectedLog}
    <LogSheet log={selectedLog} bind:open={logSidebarOpen} />
  {/if}
</main>
