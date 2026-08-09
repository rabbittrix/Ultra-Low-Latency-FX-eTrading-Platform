/**
 * Open an external URL in the system browser (Tauri) or a new tab (web preview).
 */

export async function openExternalUrl(url: string): Promise<void> {
  try {
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl(url);
    return;
  } catch {
    /* Not running inside Tauri, or opener unavailable. */
  }
  window.open(url, '_blank', 'noopener,noreferrer');
}
