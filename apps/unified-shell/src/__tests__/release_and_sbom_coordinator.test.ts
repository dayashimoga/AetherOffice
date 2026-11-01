import { describe, it, expect, beforeEach } from "vitest";
import { ReleaseAndSbomCoordinator } from "../state/ReleaseAndSbomCoordinator";

describe("ReleaseAndSbomCoordinator (Sub-Phase 53.2.2)", () => {
  let coordinator: ReleaseAndSbomCoordinator;

  beforeEach(() => {
    coordinator = new ReleaseAndSbomCoordinator();
  });

  it("test_01_default_state_initialization", () => {
    const state = coordinator.getState();
    expect(state.manifest).toBeNull();
    expect(state.cyclonedxJson).toBeNull();
    expect(state.spdxContent).toBeNull();
    expect(state.licenseAudit).toBeNull();
    expect(Object.keys(state.verifiedArtifacts).length).toBe(0);
    expect(state.isLoading).toBe(false);
    expect(state.errorMessage).toBeNull();
  });

  it("test_02_fetch_manifest_populates_5_platforms", async () => {
    const manifest = await coordinator.fetchManifest();
    expect(manifest.app_name).toBe("AetherOffice");
    expect(manifest.version).toBe("0.1.0");
    expect(manifest.artifacts.length).toBeGreaterThanOrEqual(9);

    const state = coordinator.getState();
    expect(state.manifest).toEqual(manifest);
    expect(state.isLoading).toBe(false);

    // Verify all 5 operating systems are represented in the artifacts
    const platforms = new Set(manifest.artifacts.map((a) => a.platform));
    expect(platforms.has("Windows")).toBe(true);
    expect(platforms.has("Linux")).toBe(true);
    expect(platforms.has("MacOS")).toBe(true);
    expect(platforms.has("Android")).toBe(true);
    expect(platforms.has("Ios")).toBe(true);
  });

  it("test_03_get_artifacts_for_platform", async () => {
    await coordinator.fetchManifest();

    const winArtifacts = coordinator.getArtifactsForPlatform("Windows");
    expect(winArtifacts.length).toBeGreaterThanOrEqual(3);
    const winFormats = winArtifacts.map((a) => a.bundle_format);
    expect(winFormats).toContain("Msi");
    expect(winFormats).toContain("Nsis");

    const linuxArtifacts = coordinator.getArtifactsForPlatform("Linux");
    expect(linuxArtifacts.length).toBeGreaterThanOrEqual(3);
    const linuxFormats = linuxArtifacts.map((a) => a.bundle_format);
    expect(linuxFormats).toContain("AppImage");
    expect(linuxFormats).toContain("Deb");
    expect(linuxFormats).toContain("Rpm");

    const macArtifacts = coordinator.getArtifactsForPlatform("MacOS");
    expect(macArtifacts.some((a) => a.bundle_format === "Dmg")).toBe(true);

    const androidArtifacts = coordinator.getArtifactsForPlatform("Android");
    expect(androidArtifacts.some((a) => a.bundle_format === "Apk")).toBe(true);
    expect(androidArtifacts.some((a) => a.bundle_format === "Aab")).toBe(true);

    const iosArtifacts = coordinator.getArtifactsForPlatform("Ios");
    expect(iosArtifacts.some((a) => a.bundle_format === "Ipa")).toBe(true);
  });

  it("test_04_get_single_artifact_by_id", async () => {
    await coordinator.fetchManifest();

    const msi = coordinator.getArtifact("win-x64-msi");
    expect(msi).not.toBeNull();
    expect(msi?.platform).toBe("Windows");
    expect(msi?.arch).toBe("X86_64");
    expect(msi?.bundle_format).toBe("Msi");
    expect(msi?.is_signed).toBe(true);
    expect(msi?.download_url).toContain("https://");

    const nonExistent = coordinator.getArtifact("non-existent-id");
    expect(nonExistent).toBeNull();
  });

  it("test_05_generate_cyclonedx_json", async () => {
    const jsonStr = await coordinator.generateCycloneDx();
    expect(jsonStr).toContain("CycloneDX");
    expect(jsonStr).toContain("1.5");

    const parsed = JSON.parse(jsonStr);
    expect(parsed.bomFormat).toBe("CycloneDX");
    expect(parsed.specVersion).toBe("1.5");
    expect(parsed.serialNumber).toContain("urn:uuid:");

    const state = coordinator.getState();
    expect(state.cyclonedxJson).toBe(jsonStr);
  });

  it("test_06_generate_spdx_json", async () => {
    const jsonStr = await coordinator.generateSpdx("json");
    expect(jsonStr).toContain("SPDX-2.3");
    expect(jsonStr).toContain("CC0-1.0");

    const parsed = JSON.parse(jsonStr);
    expect(parsed.spdxVersion).toBe("SPDX-2.3");
    expect(parsed.dataLicense).toBe("CC0-1.0");
    expect(parsed.packages.length).toBeGreaterThan(0);

    const state = coordinator.getState();
    expect(state.spdxContent).toBe(jsonStr);
  });

  it("test_07_generate_spdx_tag_value", async () => {
    const tagValue = await coordinator.generateSpdx("tag-value");
    expect(tagValue.startsWith("SPDXVersion: SPDX-2.3")).toBe(true);
    expect(tagValue).toContain("DataLicense: CC0-1.0");
    expect(tagValue).toContain("DocumentName: AetherOffice-v0.1.0");

    const state = coordinator.getState();
    expect(state.spdxContent).toBe(tagValue);
  });

  it("test_08_run_license_audit_clean_room", async () => {
    const report = await coordinator.runLicenseAudit();
    expect(report.is_clean_room_compliant).toBe(true);
    expect(report.has_gpl_contamination).toBe(false);
    expect(report.compliant_count).toBeGreaterThanOrEqual(18);
    expect(report.non_compliant_count).toBe(0);
    expect(report.unlicensed_count).toBe(0);
    expect(report.violation_details.length).toBe(0);

    const state = coordinator.getState();
    expect(state.licenseAudit).toEqual(report);
  });

  it("test_09_verify_artifact_checksum_success_and_state", async () => {
    await coordinator.fetchManifest();

    // Verify by known matching SHA-256 hex string
    const hex = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const verifiedHex = await coordinator.verifyArtifact("win-x64-msi", undefined, hex);
    expect(verifiedHex).toBe(true);

    const state = coordinator.getState();
    expect(state.verifiedArtifacts["win-x64-msi"]).toBe(true);

    // Verify empty payload bytes matching hash
    const verifiedPayload = await coordinator.verifyArtifact("win-x64-msi", new Uint8Array([]));
    expect(verifiedPayload).toBe(true);
  });

  it("test_10_verify_artifact_checksum_mismatch_and_subscription", async () => {
    await coordinator.fetchManifest();

    let notificationCount = 0;
    const unsub = coordinator.subscribe((_st) => {
      notificationCount++;
    });

    const isVerified = await coordinator.verifyArtifact(
      "win-x64-msi",
      undefined,
      "0000000000000000000000000000000000000000000000000000000000000000"
    );
    expect(isVerified).toBe(false);
    expect(coordinator.getState().verifiedArtifacts["win-x64-msi"]).toBe(false);
    expect(notificationCount).toBeGreaterThan(1);

    unsub();
  });
});
