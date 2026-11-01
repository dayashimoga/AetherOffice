/**
 * EntitlementCoordinator.ts
 * GEC-v7.0 Stage 53 (Sub-Phase 53.1.2)
 *
 * Presentation Shell Coordinator for:
 * - Commercial vs Open-Core tier capability governance
 * - Pure-Rust offline cryptographic license token activation (HMAC-SHA256)
 * - Automatic 14-day grace period tracking and degradation avoidance
 * - Feature gating for Enterprise DLP, Copilot, WebRTC collab, and Macro debuggers
 */

import {
  IPC,
  type EditionTierDto,
  type EntitlementStatusDto,
  type FeatureEntitlementDto,
  type LicenseTokenDto,
} from "../ipc";

export {
  type EditionTierDto,
  type EntitlementStatusDto,
  type FeatureEntitlementDto,
  type LicenseTokenDto,
};

export interface EntitlementState {
  tier: EditionTierDto;
  activeLicense: LicenseTokenDto | null;
  cachedFeatureStatus: Record<string, EntitlementStatusDto>;
  isGracePeriodActive: boolean;
  gracePeriodDaysRemaining: number | null;
  errorMessage: string | null;
  lastCheckedTimestamp: number | null;
}

export type EntitlementListener = (state: Readonly<EntitlementState>) => void;

export class EntitlementCoordinator {
  private state: EntitlementState;
  private listeners = new Set<EntitlementListener>();

  constructor() {
    this.state = {
      tier: "community",
      activeLicense: null,
      cachedFeatureStatus: {},
      isGracePeriodActive: false,
      gracePeriodDaysRemaining: null,
      errorMessage: null,
      lastCheckedTimestamp: null,
    };
  }

  public getState(): Readonly<EntitlementState> {
    return this.state;
  }

  public subscribe(listener: EntitlementListener): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    for (const listener of this.listeners) {
      listener(this.state);
    }
  }

  /**
   * Initializes coordinator by querying current tier and active license from Rust backend.
   */
  public async initialize(): Promise<void> {
    try {
      const tierStr = await IPC.entitlementGetTier();
      const lic = await IPC.entitlementGetLicenseInfo();
      const tier = this.parseTier(tierStr);

      let isGrace = false;
      let graceDays: number | null = null;

      if (lic && lic.expires_at_ms) {
        const now = Date.now();
        if (now > lic.expires_at_ms) {
          isGrace = true;
          const graceEnd = lic.expires_at_ms + 14 * 24 * 60 * 60 * 1000;
          graceDays = Math.max(0, Math.floor((graceEnd - now) / (24 * 60 * 60 * 1000)));
        }
      }

      this.state = {
        ...this.state,
        tier,
        activeLicense: lic,
        isGracePeriodActive: isGrace,
        gracePeriodDaysRemaining: graceDays,
        errorMessage: null,
        lastCheckedTimestamp: Date.now(),
      };
      this.notify();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state = { ...this.state, errorMessage: msg };
      this.notify();
    }
  }

  /**
   * Evaluates feature entitlement against Rust backend with caching.
   */
  public async checkFeature(
    feature: FeatureEntitlementDto | string,
    nowMs?: number
  ): Promise<EntitlementStatusDto> {
    try {
      const status = await IPC.entitlementCheckFeature(feature, nowMs);
      const isGrace = status.status === "grace-period";
      const graceDays = isGrace ? status.days_remaining : this.state.gracePeriodDaysRemaining;

      this.state = {
        ...this.state,
        cachedFeatureStatus: {
          ...this.state.cachedFeatureStatus,
          [feature]: status,
        },
        isGracePeriodActive: isGrace || this.state.isGracePeriodActive,
        gracePeriodDaysRemaining: graceDays,
        errorMessage: null,
        lastCheckedTimestamp: nowMs ?? Date.now(),
      };
      this.notify();
      return status;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      const deniedStatus: EntitlementStatusDto = {
        status: "denied",
        required_tier: "enterprise",
        reason: msg,
      };
      this.state = {
        ...this.state,
        errorMessage: msg,
      };
      this.notify();
      return deniedStatus;
    }
  }

  /**
   * Synchronous check for whether a feature is known to be granted.
   */
  public isFeatureGranted(feature: FeatureEntitlementDto | string): boolean {
    const cached = this.state.cachedFeatureStatus[feature];
    if (cached) {
      return cached.status === "granted" || cached.status === "grace-period";
    }

    // Fallback based on tier hierarchy
    const requiredMap: Record<string, EditionTierDto> = {
      "basic-editing": "community",
      "advanced-typography": "community",
      "standard-formulas": "community",
      "full-document-io": "community",
      "local-ai-copilot": "professional",
      "macro-debugger": "professional",
      "batch-conversion": "professional",
      "enterprise-dlp": "enterprise",
      "sanitization-certificates": "enterprise",
      "tamper-evident-audit-logs": "enterprise",
      "realtime-collab-webrtc": "enterprise",
      "custom-wasm-plugins": "enterprise",
    };
    const req = requiredMap[feature] ?? "community";
    const rank: Record<EditionTierDto, number> = {
      community: 1,
      professional: 2,
      enterprise: 3,
    };
    return rank[this.state.tier] >= rank[req];
  }

  /**
   * Applies cryptographically signed license token.
   */
  public async applyLicense(
    token: LicenseTokenDto,
    secretKey: Uint8Array | number[],
    nowMs?: number
  ): Promise<EntitlementStatusDto> {
    try {
      const res = await IPC.entitlementApplyLicense(token, secretKey, nowMs);
      const tierStr = await IPC.entitlementGetTier();
      const lic = await IPC.entitlementGetLicenseInfo();
      const tier = this.parseTier(tierStr);

      const isGrace = res.status === "grace-period";
      const graceDays = isGrace ? res.days_remaining : null;

      this.state = {
        ...this.state,
        tier,
        activeLicense: lic ?? token,
        cachedFeatureStatus: {},
        isGracePeriodActive: isGrace,
        gracePeriodDaysRemaining: graceDays,
        errorMessage: null,
        lastCheckedTimestamp: nowMs ?? Date.now(),
      };
      this.notify();
      return res;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state = {
        ...this.state,
        errorMessage: msg,
      };
      this.notify();
      throw err;
    }
  }

  /**
   * Deactivates license token and resets system to Community open-core tier.
   */
  public async removeLicense(): Promise<boolean> {
    try {
      await IPC.entitlementRemoveLicense();
      const tierStr = await IPC.entitlementGetTier();
      const tier = this.parseTier(tierStr);

      this.state = {
        ...this.state,
        tier,
        activeLicense: null,
        cachedFeatureStatus: {},
        isGracePeriodActive: false,
        gracePeriodDaysRemaining: null,
        errorMessage: null,
        lastCheckedTimestamp: Date.now(),
      };
      this.notify();
      return true;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state = { ...this.state, errorMessage: msg };
      this.notify();
      return false;
    }
  }

  /**
   * Sets manual tier override (e.g. for development, testing or demo purposes).
   */
  public async setTierOverride(tier: EditionTierDto): Promise<string> {
    try {
      const res = await IPC.entitlementSetTier(tier);
      const parsed = this.parseTier(res);

      this.state = {
        ...this.state,
        tier: parsed,
        cachedFeatureStatus: {},
        errorMessage: null,
        lastCheckedTimestamp: Date.now(),
      };
      this.notify();
      return res;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state = { ...this.state, errorMessage: msg };
      this.notify();
      throw err;
    }
  }

  public canAccessProfessionalFeatures(): boolean {
    return this.state.tier === "professional" || this.state.tier === "enterprise";
  }

  public canAccessEnterpriseFeatures(): boolean {
    return this.state.tier === "enterprise";
  }

  public getDaysUntilExpiration(nowMs?: number): number | null {
    if (!this.state.activeLicense || !this.state.activeLicense.expires_at_ms) {
      return null; // Perpetual or no license
    }
    const now = nowMs ?? Date.now();
    const diffMs = this.state.activeLicense.expires_at_ms - now;
    if (diffMs <= 0) {
      return 0;
    }
    return Math.floor(diffMs / (24 * 60 * 60 * 1000));
  }

  private parseTier(s: string): EditionTierDto {
    const clean = s.trim().toLowerCase();
    if (clean === "enterprise" || clean === "ent") return "enterprise";
    if (clean === "professional" || clean === "pro") return "professional";
    return "community";
  }
}

