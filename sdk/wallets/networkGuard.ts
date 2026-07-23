/**
 * Configuration environment definition mapping.
 */
export const APP_STELLAR_NETWORK = process.env.NEXT_PUBLIC_STELLAR_NETWORK?.toUpperCase() || 'TESTNET'; // Default safety fallback

export class NetworkMismatchError extends Error {
  constructor(public expectedNetwork: string, public actualNetwork: string) {
    super(`Network mismatch detected. Please switch your wallet to ${expectedNetwork}.`);
    this.name = 'NetworkMismatchError';
  }
}

/**
 * Validates the extension provider's current network state against the core application configuration profile.
 * @returns {Promise<boolean>} True if networks align cleanly.
 * @throws {NetworkMismatchError} If the extension configuration points to a different ledger tier.
 */
export const validateWalletNetwork = async (getWalletNetworkCallback: () => Promise<string>): Promise<boolean> => {
  try {
    // Fetch the active network setting from the connected wallet extension (e.g., Freighter)
    const walletNetworkRaw = await getWalletNetworkCallback();
    const walletNetwork = walletNetworkRaw.toUpperCase();

    // Task Requirement: Check the wallet's active network against the app's STELLAR_NETWORK
    if (walletNetwork !== APP_STELLAR_NETWORK) {
      throw new NetworkMismatchError(APP_STELLAR_NETWORK, walletNetwork);
    }

    return true;
  } catch (error) {
    if (error instanceof NetworkMismatchError) throw error;
    throw new Error(`Failed to query wallet network status: ${error instanceof Error ? error.message : 'Unknown exception'}`);
  }
};