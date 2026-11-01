import { describe, it, expect, beforeEach } from "vitest";
import {
  EntitlementCoordinator,
  type LicenseTokenDto,
  type EntitlementState,
} from "../state/EntitlementCoordinator";

const ONE_DAY_MS = 24 * 60 * 60 * 1000;
const SECRET_KEY = [1, 2, 3, 4, 5, 6, 7, 8];

describe("EntitlementCoordinator (Sub-Phase 53.1.2)", () => {
  let coordinator: EntitlementCoordinator;

  beforeEach(async () => {
    coordinator = new EntitlementCoordinator();
    await coordinator.removeLicense();
  });

  it("test_01_default_initialization_community_tier", async () => {
    await coordinator.initialize();
    const state = coordinator.getState();

    expect(state.tier).toBe("community");
    expect(state.activeLicense).toBeNull();
    expect(state.isGracePeriodActive).toBe(false);
    expect(state.gracePeriodDaysRemaining).toBeNull();
    expect(state.errorMessage).toBeNull();
    expect(coordinator.canAccessProfessionalFeatures()).toBe(false);
    expect(coordinator.canAccessEnterpriseFeatures()).toBe(false);
  });

  it("test_02_open_core_features_granted_by_default", async () => {
    await coordinator.initialize();

    const basicEdit = await coordinator.checkFeature("basic-editing");
    expect(basicEdit.status).toBe("granted");
    expect(coordinator.isFeatureGranted("basic-editing")).toBe(true);

    const typography = await coordinator.checkFeature("advanced-typography");
    expect(typography.status).toBe("granted");

    const formulas = await coordinator.checkFeature("standard-formulas");
    expect(formulas.status).toBe("granted");

    const fullIo = await coordinator.checkFeature("full-document-io");
    expect(fullIo.status).toBe("granted");
  });

  it("test_03_commercial_features_denied_in_community_tier", async () => {
    await coordinator.initialize();

    const copilot = await coordinator.checkFeature("local-ai-copilot");
    expect(copilot.status).toBe("denied");
    if (copilot.status === "denied") {
      expect(copilot.required_tier).toBe("professional");
    }

    const dlp = await coordinator.checkFeature("enterprise-dlp");
    expect(dlp.status).toBe("denied");
    if (dlp.status === "denied") {
      expect(dlp.required_tier).toBe("enterprise");
    }

    const collab = await coordinator.checkFeature("realtime-collab-webrtc");
    expect(collab.status).toBe("denied");
  });

  it("test_04_tier_override_professional_gating", async () => {
    await coordinator.setTierOverride("professional");
    const state = coordinator.getState();

    expect(state.tier).toBe("professional");
    expect(coordinator.canAccessProfessionalFeatures()).toBe(true);
    expect(coordinator.canAccessEnterpriseFeatures()).toBe(false);

    const copilot = await coordinator.checkFeature("local-ai-copilot");
    expect(copilot.status).toBe("granted");

    const macro = await coordinator.checkFeature("macro-debugger");
    expect(macro.status).toBe("granted");

    const dlp = await coordinator.checkFeature("enterprise-dlp");
    expect(dlp.status).toBe("denied");
  });

  it("test_05_tier_override_enterprise_full_access", async () => {
    await coordinator.setTierOverride("enterprise");
    expect(coordinator.canAccessEnterpriseFeatures()).toBe(true);
    expect(coordinator.canAccessProfessionalFeatures()).toBe(true);

    const allFeatures = [
      "basic-editing",
      "advanced-typography",
      "standard-formulas",
      "full-document-io",
      "local-ai-copilot",
      "macro-debugger",
      "batch-conversion",
      "enterprise-dlp",
      "sanitization-certificates",
      "tamper-evident-audit-logs",
      "realtime-collab-webrtc",
      "custom-wasm-plugins",
    ];

    for (const f of allFeatures) {
      const res = await coordinator.checkFeature(f);
      expect(res.status).toBe("granted");
      expect(coordinator.isFeatureGranted(f)).toBe(true);
    }
  });

  it("test_06_perpetual_license_application", async () => {
    const token: LicenseTokenDto = {
      license_id: "LIC-ENT-2026-PERPETUAL",
      licensee_name: "MegaCorp Industries",
      organization: "MegaCorp",
      tier: "enterprise",
      issued_at_ms: Date.now(),
      expires_at_ms: null,
      max_seats: 1000,
      custom_features: [],
      signature: "valid-mock-hmac-sha256-signature",
    };

    const res = await coordinator.applyLicense(token, SECRET_KEY);
    expect(res.status).toBe("granted");

    const state = coordinator.getState();
    expect(state.tier).toBe("enterprise");
    expect(state.activeLicense?.license_id).toBe("LIC-ENT-2026-PERPETUAL");
    expect(state.isGracePeriodActive).toBe(false);
    expect(coordinator.getDaysUntilExpiration()).toBeNull();

    const dlp = await coordinator.checkFeature("enterprise-dlp");
    expect(dlp.status).toBe("granted");
  });

  it("test_07_expired_license_in_grace_period", async () => {
    const issued = 1_700_000_000_000;
    const expires = issued + 30 * ONE_DAY_MS;
    // 5 days past expiration
    const nowMs = expires + 5 * ONE_DAY_MS;

    const token: LicenseTokenDto = {
      license_id: "LIC-PRO-GRACE",
      licensee_name: "Startup Team",
      organization: "Startup Inc",
      tier: "professional",
      issued_at_ms: issued,
      expires_at_ms: expires,
      max_seats: 10,
      custom_features: [],
      signature: "valid-sig-grace",
    };

    const res = await coordinator.applyLicense(token, SECRET_KEY, nowMs);
    expect(res.status).toBe("grace-period");
    if (res.status === "grace-period") {
      expect(res.days_remaining).toBe(9); // 14 - 5 = 9
      expect(res.message).toContain("grace period");
    }

    const state = coordinator.getState();
    expect(state.tier).toBe("professional");
    expect(state.isGracePeriodActive).toBe(true);
    expect(state.gracePeriodDaysRemaining).toBe(9);
    expect(coordinator.getDaysUntilExpiration(nowMs)).toBe(0);

    const copilot = await coordinator.checkFeature("local-ai-copilot", nowMs);
    expect(copilot.status).toBe("grace-period");
  });

  it("test_08_expired_license_beyond_grace_period_rejected", async () => {
    const issued = 1_700_000_000_000;
    const expires = issued + 30 * ONE_DAY_MS;
    // 25 days past expiration (exceeds 14 days grace)
    const nowMs = expires + 25 * ONE_DAY_MS;

    const token: LicenseTokenDto = {
      license_id: "LIC-EXPIRED-TOTALLY",
      licensee_name: "Defunct Co",
      organization: null,
      tier: "enterprise",
      issued_at_ms: issued,
      expires_at_ms: expires,
      max_seats: 5,
      custom_features: [],
      signature: "valid-sig-expired",
    };

    await expect(coordinator.applyLicense(token, SECRET_KEY, nowMs)).rejects.toThrow(
      "License expired beyond allowable grace period"
    );

    const state = coordinator.getState();
    expect(state.tier).toBe("community");
  });

  it("test_09_custom_feature_overrides", async () => {
    const token: LicenseTokenDto = {
      license_id: "LIC-CUSTOM-ADDONS",
      licensee_name: "Community Plus User",
      organization: null,
      tier: "community",
      issued_at_ms: Date.now(),
      expires_at_ms: Date.now() + 90 * ONE_DAY_MS,
      max_seats: 1,
      custom_features: ["local-ai-copilot", "custom-wasm-plugins"],
      signature: "valid-sig-addons",
    };

    await coordinator.applyLicense(token, SECRET_KEY);
    const state = coordinator.getState();
    expect(state.tier).toBe("community");

    const copilot = await coordinator.checkFeature("local-ai-copilot");
    expect(copilot.status).toBe("granted");

    const plugins = await coordinator.checkFeature("custom-wasm-plugins");
    expect(plugins.status).toBe("granted");

    const macro = await coordinator.checkFeature("macro-debugger");
    expect(macro.status).toBe("denied");

    const dlp = await coordinator.checkFeature("enterprise-dlp");
    expect(dlp.status).toBe("denied");
  });

  it("test_10_remove_license_and_subscription_listener", async () => {
    const states: EntitlementState[] = [];
    const unsubscribe = coordinator.subscribe((st) => {
      states.push({ ...st });
    });

    const token: LicenseTokenDto = {
      license_id: "LIC-SUB-TEST",
      licensee_name: "Observer Test",
      organization: "Test Corp",
      tier: "enterprise",
      issued_at_ms: Date.now(),
      expires_at_ms: Date.now() + 100 * ONE_DAY_MS,
      max_seats: 20,
      custom_features: [],
      signature: "valid-sig-observer",
    };

    await coordinator.applyLicense(token, SECRET_KEY);
    expect(coordinator.getState().tier).toBe("enterprise");

    await coordinator.removeLicense();
    expect(coordinator.getState().tier).toBe("community");
    expect(coordinator.getState().activeLicense).toBeNull();

    unsubscribe();
    // After unsubscribe, further updates do not trigger listener
    const beforeCount = states.length;
    await coordinator.setTierOverride("professional");
    expect(states.length).toBe(beforeCount);
  });
});

