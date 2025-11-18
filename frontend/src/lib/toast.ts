/**
 * Global toast notification utility
 * Can be used from non-React code (e.g., API functions)
 */

type ToastType = "success" | "error" | "warning" | "info";

type ToastCallback = (message: string, type: ToastType) => void;

let toastCallback: ToastCallback | null = null;

/**
 * Register the toast callback from the ToastProvider
 * This should be called once when the ToastProvider mounts
 */
export function registerToastCallback(callback: ToastCallback) {
  toastCallback = callback;
}

/**
 * Unregister the toast callback
 * This should be called when the ToastProvider unmounts
 */
export function unregisterToastCallback() {
  toastCallback = null;
}

/**
 * Show a toast notification
 * Can be called from anywhere, including non-React code
 */
export function showToast(message: string, type: ToastType = "info") {
  if (toastCallback) {
    toastCallback(message, type);
  } else {
    // Fallback to console if toast system isn't initialized yet
    console.warn(`[Toast] ${type.toUpperCase()}: ${message}`);
  }
}

/**
 * Check if an error is a network error and show toast
 */
export function handleNetworkError(error: unknown): boolean {
  if (
    error instanceof TypeError &&
    (error.message.includes("fetch") ||
      error.message.includes("Failed to fetch") ||
      error.message.includes("NetworkError"))
  ) {
    showToast("Network error: Unable to connect to the server. Please check your connection and try again.", "error");
    return true;
  }
  return false;
}

