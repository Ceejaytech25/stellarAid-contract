import {
  isBrowser,
  isConnected,
  requestAccess,
  signTransaction,
} from "@stellar/freighter-api";

export const FREIGHTER_DOWNLOAD_URL = "https://www.freighter.app/";

export const NETWORK_PASSPHRASE = {
  MAINNET: "Public Global Stellar Network ; September 2015",
  TESTNET: "Test SDF Network ; September 2015",
} as const;

export type StellarNetwork = (typeof NETWORK_PASSPHRASE)[keyof typeof NETWORK_PASSPHRASE];

/** Thrown when Freighter extension is not present in the browser. */
export class FreighterNotInstalledError extends Error {
  readonly downloadUrl = FREIGHTER_DOWNLOAD_URL;

  constructor() {
    super(
      `Freighter wallet extension is not installed. ` +
        `Download it at ${FREIGHTER_DOWNLOAD_URL}`
    );
    this.name = "FreighterNotInstalledError";
  }
}

/**
 * Return true when the Freighter extension is detectable in the current
 * environment. Synchronous — no network or extension round-trip.
 *
 * Note: use this to gate UI (e.g. show a "Connect Freighter" button only when
 * true). For the authoritative check before signing, rely on the errors thrown
 * by connectFreighter() and signTransactionWithFreighter().
 */
export function isFreighterInstalled(): boolean {
  return isBrowser && "freighter" in window;
}

/**
 * Ask Freighter to expose the user's public key to this page.
 * Opens the Freighter popup for approval on first call; subsequent calls
 * return immediately if the site is already trusted.
 *
 * @returns The connected Stellar public key (G…).
 * @throws {FreighterNotInstalledError} when the extension is absent.
 * @throws {Error} when the user rejects the connection request.
 */
export async function connectFreighter(): Promise<string> {
  if (!isFreighterInstalled()) {
    throw new FreighterNotInstalledError();
  }

  const connectionCheck = await isConnected();
  if (connectionCheck.error) {
    throw new Error(
      `Freighter connection check failed: ${connectionCheck.error.message}`
    );
  }

  // requestAccess opens the Freighter popup and returns the public key once
  // the user grants permission (or immediately if already allowed).
  const { address, error } = await requestAccess();
  if (error) {
    throw new Error(`Freighter access denied: ${error.message}`);
  }
  if (!address) {
    throw new Error(
      "Freighter did not return a public key. Make sure the extension is unlocked."
    );
  }

  return address;
}

/**
 * Sign a base64-encoded XDR transaction envelope with Freighter.
 *
 * @param xdr     - Base64-encoded XDR of the unsigned transaction envelope.
 * @param network - Network passphrase. Use NETWORK_PASSPHRASE.TESTNET or
 *                  NETWORK_PASSPHRASE.MAINNET, or a custom passphrase string.
 * @returns Signed XDR envelope as a base64 string.
 * @throws {FreighterNotInstalledError} when the extension is absent.
 * @throws {Error} when the user rejects signing or Freighter reports an error.
 */
export async function signTransactionWithFreighter(
  xdr: string,
  network: string
): Promise<string> {
  if (!isFreighterInstalled()) {
    throw new FreighterNotInstalledError();
  }

  const { signedTxXdr, error } = await signTransaction(xdr, {
    networkPassphrase: network,
  });

  if (error) {
    throw new Error(`Freighter signing failed: ${error.message}`);
  }
  if (!signedTxXdr) {
    throw new Error(
      "Freighter did not return signed XDR. The request may have been rejected."
    );
  }

  return signedTxXdr;
}
