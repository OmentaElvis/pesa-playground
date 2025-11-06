import { getAppInfo } from '$lib/api';

export async function load() {
  try {
    const appInfo = await getAppInfo();
    return {
      appInfo
    };
  } catch (e) {
    console.error("Failed to fetch app info:", e);
    return {
      appInfo: {
        name: 'Error',
        version: 'Error',
        description: 'Could not fetch app info from backend.',
        authors: 'Error'
      }
    };
  }
}
